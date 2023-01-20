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
            let data = message.leak();
            match serde_json::from_str::<'_, CloudEvent>(data) {
                Ok(event) => {
                    let event_session_id = event.session_id();
                    if session_id.is_none() || session_id != event_session_id {
                        let event_source = event.source();
                        if source.filter(|&source| source != event_source).is_none() {
                            let event_topic = event.topic();
                            if topic.filter(|&topic| topic != event_topic).is_none() {
                                let message = Message::Text(data.to_string());
                                if let Err(err) = socket.send(message).await {
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
