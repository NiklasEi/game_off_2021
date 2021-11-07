use futures::{lock::Mutex, stream::SplitSink, StreamExt};
use log::{error, info};
use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    sync::Arc,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{
    ws::{Message, WebSocket},
    Error, Filter, Rejection, Reply,
};

pub mod matchbox {
    use serde::{Deserialize, Serialize};

    pub type PeerId = String;

    /// Requests go from peer to signalling server
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    pub enum PeerRequest<S> {
        Uuid(PeerId),
        Signal { receiver: PeerId, data: S },
    }

    /// Events go from signalling server to peer
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    pub enum PeerEvent<S> {
        NewPeer(PeerId),
        Signal { sender: PeerId, data: S },
    }
}
use matchbox::*;

type PeerRequest = matchbox::PeerRequest<serde_json::Value>;
type PeerEvent = matchbox::PeerEvent<serde_json::Value>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RequestedRoom {
    Id(String),
    Next(usize),
}

pub(crate) struct Peer {
    pub uuid: PeerId,
    pub room: RequestedRoom,
    pub sender:
        Option<tokio::sync::mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

#[derive(Default)]
pub(crate) struct State {
    clients: HashMap<PeerId, Peer>,
    next_rooms: HashMap<usize, HashSet<PeerId>>,
    id_rooms: HashMap<String, HashSet<PeerId>>,
}

impl State {
    /// Returns peers already in room
    fn add_peer(&mut self, peer: Peer) -> Vec<PeerId> {
        let peer_id = peer.uuid.clone();
        let room = peer.room.clone();
        self.clients.insert(peer.uuid.clone(), peer);

        match room {
            RequestedRoom::Id(room_id) => {
                let peers = self.id_rooms.entry(room_id).or_default();
                let ret = peers.iter().cloned().collect();
                peers.insert(peer_id);
                ret
            }
            RequestedRoom::Next(num_players) => {
                let peers = self.next_rooms.entry(num_players).or_default();
                let ret = peers.iter().cloned().collect();
                if peers.len() == num_players - 1 {
                    peers.clear() // the room is complete, we can forget about it now
                } else {
                    peers.insert(peer_id);
                }
                ret
            }
        }
    }

    fn remove_peer(&mut self, peer_id: &PeerId) {
        let peer = self
            .clients
            .remove(peer_id)
            .expect("Couldn't find uuid to remove");

        let room_peers = match peer.room {
            RequestedRoom::Id(room_id) => self.id_rooms.get_mut(&room_id),
            RequestedRoom::Next(num_players) => self.next_rooms.get_mut(&num_players),
        };

        if let Some(room_peers) = room_peers {
            room_peers.remove(peer_id);
        }
    }

    fn try_send(&self, id: &PeerId, message: Message) {
        let peer = self.clients.get(id);
        let peer = match peer {
            Some(peer) => peer,
            None => {
                error!("Unknown peer {:?}", id);
                return;
            }
        };
        if let Err(e) = peer.sender.as_ref().unwrap().send(Ok(message.clone())) {
            error!("Error sending message {:?}", e);
        }
    }
}

fn parse_room_id(id: String) -> RequestedRoom {
    match id.strip_prefix("next_").and_then(|n| n.parse().ok()) {
        Some(num_players) => RequestedRoom::Next(num_players),
        None => RequestedRoom::Id(id),
    }
}

pub(crate) fn ws_filter(
    state: Arc<Mutex<State>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::ws()
        .and(warp::any())
        .and(warp::path::param().map(parse_room_id))
        .and(with_state(state.clone()))
        .and_then(ws_handler)
}

fn with_state(
    state: Arc<Mutex<State>>,
) -> impl Filter<Extract = (Arc<Mutex<State>>,), Error = Infallible> + Clone {
    warp::any().map(move || state.clone())
}

pub(crate) async fn ws_handler(
    ws: warp::ws::Ws,
    requested_room: RequestedRoom,
    state: Arc<Mutex<State>>,
) -> std::result::Result<impl Reply, Rejection> {
    Ok(ws.on_upgrade(move |websocket| handle_ws(websocket, state, requested_room)))
}

#[derive(Debug, thiserror::Error)]
enum RequestError {
    #[error("Warp error")]
    WarpError(#[from] warp::Error),
    #[error("Text error")]
    TextError,
    #[error("Json error")]
    JsonError(#[from] serde_json::Error),
}

fn parse_request(request: Result<Message, Error>) -> Result<PeerRequest, RequestError> {
    let request = request?;

    if !request.is_text() {
        return Err(RequestError::TextError);
    }

    let request = request.to_str().map_err(|_| RequestError::TextError)?;

    let request: PeerRequest = serde_json::from_str(request)?;

    Ok(request)
}

fn spawn_sender_task(
    sender: SplitSink<WebSocket, Message>,
) -> mpsc::UnboundedSender<std::result::Result<Message, warp::Error>> {
    let (client_sender, receiver) = mpsc::unbounded_channel();
    tokio::task::spawn(UnboundedReceiverStream::new(receiver).forward(sender));
    client_sender
}

async fn handle_ws(websocket: WebSocket, state: Arc<Mutex<State>>, requested_room: RequestedRoom) {
    let (ws_sender, mut ws_receiver) = websocket.split();
    let sender = spawn_sender_task(ws_sender);
    let mut peer_uuid = None;

    while let Some(request) = ws_receiver.next().await {
        let request = match parse_request(request) {
            Ok(request) => request,
            Err(RequestError::WarpError(e)) => {
                error!("Warp error while receiving request: {:?}", e);
                // Most likely a ConnectionReset or similar.
                // just give up on this peer.
                break;
            }
            Err(e) => {
                error!("Error untangling request: {:?}", e);
                continue;
            }
        };

        info!("{:?} <- {:?}", peer_uuid, request);

        match request {
            PeerRequest::Uuid(id) => {
                if peer_uuid.is_some() {
                    error!("client set uuid more than once");
                    continue;
                }
                peer_uuid = Some(id.clone());

                let mut state = state.lock().await;
                let peers = state.add_peer(Peer {
                    uuid: id.clone(),
                    sender: Some(sender.clone()),
                    room: requested_room.clone(),
                });

                let event = Message::text(
                    serde_json::to_string(&PeerEvent::NewPeer(id.clone()))
                        .expect("error serializing message"),
                );

                for peer_id in peers {
                    // Tell everyone about this new peer
                    info!("{:?} -> {:?}", peer_id, event.to_str().unwrap());
                    state.try_send(&peer_id, event.clone());
                }
            }
            PeerRequest::Signal { receiver, data } => {
                let sender = match peer_uuid.clone() {
                    Some(sender) => sender,
                    None => {
                        error!("client is trying signal before sending uuid");
                        continue;
                    }
                };
                let event = Message::text(
                    serde_json::to_string(&PeerEvent::Signal { sender, data })
                        .expect("error serializing message"),
                );
                let state = state.lock().await;
                if let Err(e) = state
                    .clients
                    .get(&receiver)
                    .unwrap()
                    .sender
                    .as_ref()
                    .unwrap()
                    .send(Ok(event))
                {
                    error!("error sending: {:?}", e);
                }
            }
        }
    }

    info!("Removing peer: {:?}", peer_uuid);
    if let Some(uuid) = peer_uuid {
        let mut state = state.lock().await;
        state.remove_peer(&uuid);
    }
}

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use futures::pin_mut;
    use tokio::{select, time};
    use warp::{test::WsClient, ws::Message, Filter, Rejection, Reply};

    use crate::signaling::{parse_room_id, PeerEvent, RequestedRoom};

    fn api() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        super::ws_filter(Default::default())
    }

    #[tokio::test]
    async fn ws_connect() {
        let _ = pretty_env_logger::try_init();
        let api = api();

        // let req = warp::test::ws().path("/echo");
        warp::test::ws()
            .path("/room_a")
            .handshake(api)
            // .handshake(ws_echo())
            .await
            .expect("handshake");
    }

    #[tokio::test]
    async fn new_peer() {
        let _ = pretty_env_logger::try_init();
        let api = api();

        // let req = warp::test::ws().path("/echo");
        let mut client_a = warp::test::ws()
            .path("/room_a")
            .handshake(api.clone())
            // .handshake(ws_echo())
            .await
            .expect("handshake");

        client_a
            .send(Message::text(r#"{"Uuid": "uuid-a"}"#.to_string()))
            .await;

        let mut client_b = warp::test::ws()
            .path("/room_a")
            .handshake(api)
            // .handshake(ws_echo())
            .await
            .expect("handshake");

        client_b
            .send(Message::text(r#"{"Uuid": "uuid-b"}"#.to_string()))
            .await;

        let a_msg = client_a.recv().await;
        let new_peer_event: PeerEvent =
            serde_json::from_str(a_msg.unwrap().to_str().unwrap()).unwrap();

        assert_eq!(new_peer_event, PeerEvent::NewPeer("uuid-b".to_string()));
    }

    #[tokio::test]
    async fn signal() {
        let _ = pretty_env_logger::try_init();
        let api = api();

        // let req = warp::test::ws().path("/echo");
        let mut client_a = warp::test::ws()
            .path("/room_a")
            .handshake(api.clone())
            // .handshake(ws_echo())
            .await
            .expect("handshake");

        client_a
            .send(Message::text(r#"{"Uuid": "uuid-a"}"#.to_string()))
            .await;

        let mut client_b = warp::test::ws()
            .path("/room_a")
            .handshake(api)
            // .handshake(ws_echo())
            .await
            .expect("handshake");

        client_b
            .send(Message::text(r#"{"Uuid": "uuid-b"}"#.to_string()))
            .await;

        let a_msg = client_a.recv().await;
        let new_peer_event: PeerEvent =
            serde_json::from_str(a_msg.unwrap().to_str().unwrap()).unwrap();

        let peer_uuid = match new_peer_event {
            PeerEvent::NewPeer(peer) => peer,
            _ => panic!("unexpected event"),
        };

        client_a
            .send(Message::text(format!(
                "{{\"Signal\": {{\"receiver\": \"{}\", \"data\": \"123\" }}}}",
                peer_uuid
            )))
            .await;

        let b_msg = client_b.recv().await;
        let signal_event: PeerEvent =
            serde_json::from_str(b_msg.unwrap().to_str().unwrap()).unwrap();

        assert_eq!(
            signal_event,
            PeerEvent::Signal {
                data: serde_json::Value::String("123".to_string()),
                sender: "uuid-a".to_string(),
            }
        );
    }

    async fn recv_peer_event(client: &mut WsClient) -> PeerEvent {
        let message = client.recv().await;
        serde_json::from_str(message.unwrap().to_str().unwrap()).unwrap()
    }

    #[tokio::test]
    async fn match_pairs() {
        let _ = pretty_env_logger::try_init();
        let api = api();

        let mut client_a = warp::test::ws()
            .path("/next_2")
            .handshake(api.clone())
            // .handshake(ws_echo())
            .await
            .expect("handshake");

        client_a
            .send(Message::text(r#"{"Uuid": "uuid-a"}"#.to_string()))
            .await;

        let mut client_b = warp::test::ws()
            .path("/next_2")
            .handshake(api.clone())
            // .handshake(ws_echo())
            .await
            .expect("handshake");

        client_b
            .send(Message::text(r#"{"Uuid": "uuid-b"}"#.to_string()))
            .await;

        let mut client_c = warp::test::ws()
            .path("/next_2")
            .handshake(api.clone())
            // .handshake(ws_echo())
            .await
            .expect("handshake");

        client_c
            .send(Message::text(r#"{"Uuid": "uuid-c"}"#.to_string()))
            .await;

        let mut client_d = warp::test::ws()
            .path("/next_2")
            .handshake(api.clone())
            // .handshake(ws_echo())
            .await
            .expect("handshake");

        client_d
            .send(Message::text(r#"{"Uuid": "uuid-d"}"#.to_string()))
            .await;

        // Clients should be matched in pairs as they arrive, i.e. a + b and c + d
        let new_peer_b = recv_peer_event(&mut client_a).await;
        let new_peer_d = recv_peer_event(&mut client_c).await;

        assert_eq!(new_peer_b, PeerEvent::NewPeer("uuid-b".to_string()));
        assert_eq!(new_peer_d, PeerEvent::NewPeer("uuid-d".to_string()));

        let timeout = time::sleep(Duration::from_millis(100));
        pin_mut!(timeout);
        select! {
            _ = client_a.recv() => panic!("unexpected message"),
            _ = client_b.recv() => panic!("unexpected message"),
            _ = client_c.recv() => panic!("unexpected message"),
            _ = client_d.recv() => panic!("unexpected message"),
            _ = &mut timeout => {}
        }
    }

    #[test]
    fn requested_room() {
        assert_eq!(parse_room_id("next_2".into()), RequestedRoom::Next(2));
    }
}
