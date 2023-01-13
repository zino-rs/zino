use futures::stream::Stream;
use parking_lot::RwLock;
use std::{collections::HashMap, sync::LazyLock};
use tokio::sync::mpsc::{self, error::TrySendError, Receiver, Sender};
use tokio_stream::wrappers::ReceiverStream;
use zino_core::{
    application::Application,
    channel::{CloudEvent, Subscription},
    Uuid,
};

/// A emitter is a sender of cloud events.
type Emitter = Sender<CloudEvent>;

/// A listener is a receiver of cloud events.
type Listener = Receiver<CloudEvent>;

/// A subscriber of cloud events.
#[derive(Debug, Clone)]
struct Subscriber {
    /// Emitter.
    emitter: Emitter,
    /// Filter.
    filter: Option<Subscription>,
}

impl Subscriber {
    /// Creates a new instance.
    #[inline]
    fn new(emitter: Emitter, filter: Option<Subscription>) -> Self {
        Self { emitter, filter }
    }

    /// Returns a reference to the emitter.
    #[inline]
    fn emitter(&self) -> &Emitter {
        &self.emitter
    }

    /// Returns a reference to the filter.
    #[inline]
    fn filter(&self) -> Option<&Subscription> {
        self.filter.as_ref()
    }
}

/// Message channel of cloud events.
#[derive(Debug)]
pub(crate) struct MessageChannel {
    /// Sender ID.
    sender_id: Uuid,
    /// Receiver.
    receiver: Listener,
}

impl MessageChannel {
    /// Creates a new `MessageChannel`.
    pub(crate) fn new() -> Self {
        let (sender, receiver) = mpsc::channel(*CHANNEL_CAPACITY);
        let sender_id = Uuid::new_v4();
        let subscriber = Subscriber::new(sender, None);
        let mut senders = CHANNEL_SUBSCRIBERS.write();
        senders.retain(|_, subscriber| !subscriber.emitter().is_closed());
        senders.insert(sender_id, subscriber);
        Self {
            sender_id,
            receiver,
        }
    }

    /// Returns a reference to the shared `MessageChannel`.
    #[inline]
    pub(crate) fn shared() -> &'static Self {
        LazyLock::force(&SHARED_CHANNEL)
    }

    /// Attempts to send a message to all receivers in the channel except this one.
    pub(crate) fn try_send(
        &self,
        message: impl Into<CloudEvent>,
    ) -> Result<(), TrySendError<CloudEvent>> {
        let sender_id = &self.sender_id;
        let event = message.into();
        let event_source = event.source();
        let event_topic = event.topic();
        let subscribers = CHANNEL_SUBSCRIBERS.read();
        for (key, subscriber) in subscribers.iter() {
            let emitter = subscriber.emitter();
            if key != sender_id && !emitter.is_closed() {
                let is_subscribed = match subscriber.filter() {
                    Some(subscription) => {
                        subscription
                            .source()
                            .filter(|&s| s != event_source)
                            .is_none()
                            && subscription.topic().filter(|&t| t != event_topic).is_none()
                    }
                    None => true,
                };
                if is_subscribed {
                    emitter.try_send(event.clone())?;
                }
            }
        }
        Ok(())
    }

    /// Consumes `Self` and returns a message stream of `CloudEvent`.
    #[inline]
    pub(crate) fn into_stream(self) -> impl Stream<Item = CloudEvent> {
        ReceiverStream::new(self.receiver)
    }
}

impl Default for MessageChannel {
    fn default() -> Self {
        Self::new()
    }
}

/// Channel capacity.
static CHANNEL_CAPACITY: LazyLock<usize> = LazyLock::new(|| {
    let config = crate::AxumCluster::config();
    match config.get("channel") {
        Some(channel) => channel
            .as_table()
            .expect("the `channel` field should be a table")
            .get("capacity")
            .expect("the `channel.capacity` field is missing")
            .as_integer()
            .expect("the `channel.capacity` field should be an integer")
            .try_into()
            .expect("the `channel.capacity` field should be a positive integer"),
        None => 10000,
    }
});

/// Channel subscribers.
static CHANNEL_SUBSCRIBERS: LazyLock<RwLock<HashMap<Uuid, Subscriber>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Shared channel.
pub(crate) static SHARED_CHANNEL: LazyLock<MessageChannel> = LazyLock::new(MessageChannel::new);
