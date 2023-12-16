use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query,
    },
    response::IntoResponse,
};
use zino_core::channel::{CloudEvent, Subscription};

/// WebSocket endpoint handler.
pub(crate) async fn websocket_handler(
    ws: WebSocketUpgrade,
    query: Query<Subscription>,
) -> impl IntoResponse {
    ws.on_upgrade(|mut socket: WebSocket| async move {
        let subscription = query.0;
        let session_id = subscription.session_id();
        let source = subscription.source();
        let topic = subscription.topic();
        while let Some(Ok(Message::Text(message))) = socket.recv().await {
            match serde_json::from_str::<CloudEvent>(&message) {
                Ok(event) => {
                    let event_session_id = event.session_id();
                    if session_id.is_none() || session_id != event_session_id {
                        let event_source = event.source();
                        if source.filter(|&s| event_source != s).is_none() {
                            let event_type = event.event_type();
                            if topic.filter(|&t| event_type != t).is_none() {
                                if let Err(err) = socket.send(Message::Text(message)).await {
                                    tracing::error!("{err}");
                                }
                            }
                        }
                    }
                }
                Err(err) => tracing::error!("{err}"),
            }
        }
    })
}
