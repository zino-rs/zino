use axum::{
    extract::Query,
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream::Stream;
use std::convert::Infallible;
use tokio_stream::StreamExt;
use zino_core::channel::Subscription;

/// SSE endpoint handler.
pub(crate) async fn sse_handler(
    query: Query<Subscription>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let subscription = query.0;
    let session_id = subscription.session_id().map(|s| s.to_owned());
    let source = subscription.source().map(|s| s.to_owned());
    let topic = subscription.topic().map(|t| t.to_owned());
    let channel = crate::channel::axum_channel::MessageChannel::new();
    let stream = channel.into_stream().filter_map(move |event| {
        let mut sse_event_filter = None;
        let event_session_id = event.session_id();
        if session_id.is_none() || session_id.as_deref() != event_session_id {
            let event_source = event.source();
            if source.as_ref().filter(|&s| event_source != s).is_none() {
                let event_topic = event.topic();
                if topic.as_ref().filter(|&t| event_topic != t).is_none() {
                    let event_id = event.id();
                    let event_data = event.stringify_data();
                    let sse_event = Event::default()
                        .event(event_topic)
                        .data(event_data)
                        .id(event_id);
                    sse_event_filter = Some(Ok(sse_event));
                }
            }
        }
        sse_event_filter
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}
