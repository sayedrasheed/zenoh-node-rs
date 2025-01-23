use super::error::*;
use async_trait::async_trait;
use futures::{prelude::*, FutureExt};
use snafu::ResultExt;
use std::{
    collections::HashMap,
    error::Error,
    fmt::{self},
    sync::Arc,
};
use tokio::{sync::Mutex, task::JoinHandle};
use zenoh::prelude::r#async::*;
use zenoh::prelude::SplitBuffer;

use crate::session::Session;

/// Generic subscriber error
pub struct SubscriberError(pub Box<dyn Error + Send + Sync>);

/// Subscriber join handle
pub type SubscriberTaskJoinHandle = JoinHandle<Result<(), SubscriberError>>;

impl snafu::AsErrorSource for SubscriberError {
    fn as_error_source(&self) -> &(dyn Error + 'static) {
        &*self.0
    }
}

impl<E> From<E> for SubscriberError
where
    E: Error + Send + Sync + 'static,
{
    fn from(inner: E) -> Self {
        Self(Box::new(inner))
    }
}

impl fmt::Display for SubscriberError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Debug for SubscriberError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

/// Subscribe trait for subscribing to a protobuf message
#[async_trait]
pub trait Subscribe<T>: Send
where
    T: prost::Message,
{
    async fn on_data(&mut self, msg: T) -> Result<(), SubscriberError>;

    async fn on_exit(&mut self, _topic: &str) -> Result<(), SubscriberError> {
        Ok(())
    }
}

struct SubscriberBase {
    session: Session,
    subscriptions: HashMap<String, SubscriberTaskJoinHandle>,
    name: String,
}

impl SubscriberBase {
    pub fn _topics(&self) -> impl Iterator<Item = &str> {
        self.subscriptions.keys().map(|t| t.as_ref())
    }

    pub fn _name(&self) -> &str {
        &self.name
    }

    fn check_for_active_subscription(&mut self, topic: &str) -> Result<(), NodeError> {
        if let Some(handle) = self.subscriptions.get_mut(topic) {
            if handle.now_or_never().is_none() {
                return ReceiveSnafu.fail();
            } else {
                self.subscriptions.remove(topic);
            }
        }

        Ok(())
    }
}

/// Subscriber that wraps an object which implements subscribe traits for different messages
pub struct Subscriber<S: ?Sized> {
    inner: Arc<Mutex<S>>,
    base: SubscriberBase,
}

impl<S> Subscriber<S> {
    /// Creates new subscriber with given zenoh session and wrapper object
    pub fn new(session: Session, inner: S) -> Self {
        let inner = Arc::new(Mutex::new(inner));
        let inner_addr = std::ptr::addr_of!(*inner) as usize;
        let name = format!("{}:{}", std::any::type_name::<S>(), inner_addr);
        Self {
            inner,
            base: SubscriberBase {
                subscriptions: HashMap::new(),
                session,
                name,
            },
        }
    }

    /// Get inner subscriber
    pub fn get_inner(&self) -> Arc<Mutex<S>> {
        self.inner.clone()
    }

    /// Join all subscriptions for this subscriber
    pub async fn join(mut self) -> Result<S, Box<dyn Error + Send + Sync>> {
        if let Err(err) = future::try_join_all(self.base.subscriptions.drain().map(|(_, j)| {
            j.map(|r| match r {
                Ok(r) => r,
                Err(e) => Err(SubscriberError::from(e)),
            })
        }))
        .await
        {
            Err(err.0)
        } else {
            Ok(Arc::try_unwrap(self.inner).ok().unwrap().into_inner())
        }
    }
}

// Abort trait for aborting all subscriptions for this subscriber
pub trait Abort: Send {
    fn abort(&self);
}

impl<S> Abort for Subscriber<S>
where
    S: Send,
{
    // Aborts all threads subscribing to messages
    fn abort(&self) {
        for sub in &self.base.subscriptions {
            sub.1.abort();
        }
    }
}

/// Subscriber implemtation trait
#[async_trait]
pub trait SubscriberImpl<T>: Send
where
    T: prost::Message,
{
    fn name(&self) -> &str;

    async fn subscribe(&mut self, topic: &str) -> Result<(), NodeError>;
}

#[async_trait]
impl<S, T> SubscriberImpl<T> for Subscriber<S>
where
    T: prost::Message + Default + fmt::Debug + 'static,
    S: Subscribe<T> + 'static,
{
    /// Get name of subscriber
    fn name(&self) -> &str {
        &self.base.name
    }

    /// Subscribe to protobuf message with given topic
    async fn subscribe(&mut self, topic: &str) -> Result<(), NodeError> {
        self.base.check_for_active_subscription(topic)?;

        let inner = self.inner.clone();
        let handle = tokio::spawn(subscriber_task(
            inner,
            self.base.session.clone(),
            topic.to_owned(),
        ));

        self.base.subscriptions.insert(topic.to_owned(), handle);

        Ok(())
    }
}

/// Subscriber task to listen for messages
async fn subscriber_task<T>(
    inner: Arc<Mutex<dyn Subscribe<T>>>,
    session: Session,
    topic: String,
) -> Result<(), SubscriberError>
where
    T: prost::Message + Default + 'static,
{
    let session = session.into_inner();
    let receiver = session
        .declare_subscriber(topic.clone())
        .res()
        .await
        .context(DeclareReceiverSnafu);

    match receiver {
        Ok(receiver) => {
            while let Ok(sample) = receiver.recv_async().await {
                let payload = sample.payload.contiguous();
                let buf = payload.as_ref();
                match T::decode(buf) {
                    Ok(msg) => {
                        inner.lock().await.on_data(msg).await?;
                    }
                    Err(error) => return Err(SubscriberError(Box::new(error))),
                }
            }
        }
        Err(error) => {
            return Err(SubscriberError(Box::new(error)));
        }
    }

    Ok(())
}
