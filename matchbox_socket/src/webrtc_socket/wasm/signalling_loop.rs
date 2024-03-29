use crate::webrtc_socket::messages::*;
use futures::{pin_mut, FutureExt, SinkExt, StreamExt};
use futures_util::select;
use log::{debug, error};
use ws_stream_wasm::{WsMessage, WsMeta};

pub async fn signalling_loop(
    room_url: String,
    mut requests_receiver: futures_channel::mpsc::UnboundedReceiver<PeerRequest>,
    events_sender: futures_channel::mpsc::UnboundedSender<PeerEvent>,
) {
    let (_ws, mut wsio) = WsMeta::connect(&room_url, None)
        .await
        .expect("failed to connect to signalling server");

    loop {
        let next_request = requests_receiver.next().fuse();
        let next_websocket_message = wsio.next().fuse();

        pin_mut!(next_request, next_websocket_message);

        select! {
            request = next_request => {
                let request = serde_json::to_string(&request).expect("serializing request");
                debug!("-> {}", request);
                wsio.send(WsMessage::Text(request)).await.expect("request send error");
            }

            message = next_websocket_message => {
                match message {
                    Some(WsMessage::Text(message)) => {
                        debug!("{}", message);
                        let event: PeerEvent = serde_json::from_str(&message)
                            .expect(&format!("couldn't parse peer event {}", message));
                        events_sender.unbounded_send(event).unwrap();
                    },
                    Some(WsMessage::Binary(_)) => {
                        error!("Received binary data from signal server (expected text). Ignoring.");
                    },
                    None => {} // Disconnected from signalling server
                };
            }

            complete => break
        }
    }
}
