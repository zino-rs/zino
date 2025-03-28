use super::{CloudEvent, Subscription};
use ahash::{HashMap, HashMapExt};
use flume::{Receiver, SendError, Sender, TrySendError};
use futures::{Sink, Stream};
use parking_lot::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use zino_core::{LazyLock, Uuid, extension::TomlTableExt, state::State};

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

/// Message channel for sending and receiving cloud events.
#[derive(Debug, Clone)]
pub struct MessageChannel {
    /// Sender ID.
    sender_id: Uuid,
    /// Receiver.
    receiver: Listener,
}

impl MessageChannel {
    /// Creates a new instance.
    pub fn new() -> Self {
        let (sender, receiver) = flume::bounded(CHANNEL_CAPACITY.load(Relaxed));
        let sender_id = Uuid::now_v7();
        let subscriber = Subscriber::new(sender, None);
        let mut subscribers = CHANNEL_SUBSCRIBERS.write();
        subscribers.retain(|_, subscriber| !subscriber.emitter().is_disconnected());
        subscribers.insert(sender_id, subscriber);
        Self {
            sender_id,
            receiver,
        }
    }

    /// Creates a new instance with the subscription.
    pub fn with_subscription(sub: Subscription) -> Self {
        let (sender, receiver) = flume::bounded(CHANNEL_CAPACITY.load(Relaxed));
        let sender_id = Uuid::now_v7();
        let subscriber = Subscriber::new(sender, Some(sub));
        let mut subscribers = CHANNEL_SUBSCRIBERS.write();
        subscribers.retain(|_, subscriber| !subscriber.emitter().is_disconnected());
        subscribers.insert(sender_id, subscriber);
        Self {
            sender_id,
            receiver,
        }
    }

    /// Returns a reference to the shared message channel.
    #[inline]
    pub fn shared() -> &'static Self {
        &SHARED_CHANNEL
    }

    /// Get the number of subscribers that currently exist.
    #[inline]
    pub fn subscriber_count(&self) -> usize {
        CHANNEL_SUBSCRIBERS.read().len()
    }

    /// Attempts to send a message to all receivers in the channel except this one.
    /// If the channel is full or all receivers have been dropped, an error is returned.
    pub fn try_send(&self, message: impl Into<CloudEvent>) -> Result<(), TrySendError<CloudEvent>> {
        let sender_id = &self.sender_id;
        let event = message.into();
        let source = event.source();
        let event_type = event.event_type();
        let event_session_id = event.session_id();
        let subscribers = CHANNEL_SUBSCRIBERS.read();
        for (uid, subscriber) in subscribers.iter() {
            let emitter = subscriber.emitter();
            if uid != sender_id && !emitter.is_disconnected() {
                let is_subscribed = if let Some(subscription) = subscriber.filter() {
                    subscription.source().filter(|&s| source != s).is_none()
                        && subscription.topic().filter(|&t| event_type != t).is_none()
                        && subscription
                            .session_id()
                            .filter(|&s| event_session_id.is_some_and(|sid| sid != s))
                            .is_none()
                } else {
                    true
                };
                if is_subscribed {
                    emitter.try_send(event.clone())?;
                }
            }
        }
        Ok(())
    }

    /// Asynchronously sends a message to all receivers in the channel except this one,
    /// returning an error if all receivers have been dropped.
    /// If the channel is full, it will yield to the async runtime.
    pub async fn send(&self, message: impl Into<CloudEvent>) -> Result<(), SendError<CloudEvent>> {
        let sender_id = &self.sender_id;
        let event = message.into();
        let source = event.source();
        let event_type = event.event_type();
        let event_session_id = event.session_id();
        let subscribers = CHANNEL_SUBSCRIBERS.read().to_owned();
        for (uid, subscriber) in subscribers.iter() {
            let emitter = subscriber.emitter();
            if uid != sender_id && !emitter.is_disconnected() {
                let is_subscribed = if let Some(subscription) = subscriber.filter() {
                    subscription.source().filter(|&s| source != s).is_none()
                        && subscription.topic().filter(|&t| event_type != t).is_none()
                        && subscription
                            .session_id()
                            .filter(|&s| event_session_id.is_some_and(|sid| sid != s))
                            .is_none()
                } else {
                    true
                };
                if is_subscribed {
                    emitter.send_async(event.clone()).await?;
                }
            }
        }
        Ok(())
    }

    /// Returns a sink that allows asynchronously sending messages into the channel.
    pub fn sink(&self) -> impl Sink<CloudEvent> {
        let sender_id = &self.sender_id;
        if let Some(subscriber) = CHANNEL_SUBSCRIBERS.read().get(sender_id) {
            subscriber.emitter().clone().into_sink()
        } else {
            panic!("fail to get the sender `{sender_id}`");
        }
    }

    /// Returns a stream that allows asynchronously receiving messages from the channel.
    #[inline]
    pub fn stream(&self) -> impl Stream<Item = CloudEvent> + '_ {
        self.receiver.stream()
    }

    /// Converts `self` into a stream that allows asynchronously receiving messages from the channel.
    #[inline]
    pub fn into_stream(self) -> impl Stream<Item = CloudEvent> {
        self.receiver.into_stream()
    }
}

impl Default for MessageChannel {
    fn default() -> Self {
        Self::new()
    }
}

/// Channel capacity.
static CHANNEL_CAPACITY: AtomicUsize = AtomicUsize::new(10000);

/// Channel subscribers.
static CHANNEL_SUBSCRIBERS: LazyLock<RwLock<HashMap<Uuid, Subscriber>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Shared channel.
static SHARED_CHANNEL: LazyLock<MessageChannel> = LazyLock::new(|| {
    let capacity = if let Some(channel) = State::shared().config().get("channel") {
        channel
            .as_table()
            .expect("field `channel` should be a table")
            .get_usize("capacity")
            .expect("field `channel.capacity` should be a positive integer")
    } else {
        10000
    };
    CHANNEL_CAPACITY.store(capacity, Relaxed);
    MessageChannel::new()
});
