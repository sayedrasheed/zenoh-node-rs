use crate::error::NodeError;
pub use crate::publisher::Publisher;
use crate::session::Session;
pub use crate::subscriber::SubscriberImpl;
pub use crate::subscriber::{Abort, Subscribe, Subscriber, SubscriberError};

/// Node object wrapping a zenoh session
/// # Examples
///
/// ```
/// use prost::Message;
/// use snafu::Error;
/// use zenoh_node::builder::NodeBuilder;
/// use zenoh_node::node::{Abort, Subscribe, SubscriberError};

/// use async_trait::async_trait;

/// /// Sample prost protobuf message, in practice this will be generated from
/// /// a .proto file
/// #[derive(Clone, PartialEq, Message)]
/// pub struct SampleMessage {
///     #[prost(string, tag = "1")]
///     pub name: String,
/// }

/// /// My minimal subscriber
/// pub struct MySubscriber {}

/// /// Implement Subscribe trait for the SampleMessage
/// #[async_trait]
/// impl Subscribe<SampleMessage> for MySubscriber {
///     async fn on_data(&mut self, msg: SampleMessage) -> Result<(), SubscriberError> {
///         println!("Sample message received with name: {}", msg.name);
///         Ok(())
///     }
/// }

/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
///     let mut builder = NodeBuilder::new();
///     let node = builder.build().await?;

///     let publisher = node.new_publisher::<SampleMessage>("my_topic").await?;

///     let my_subscriber = MySubscriber {};
///     let mut subscriber = node.new_subscriber(my_subscriber).await?;

///     node.subscribe("my_topic", &mut subscriber).await?;

///     let sample_msg = SampleMessage {
///         name: String::from("John Doe"),
///     };

///     publisher.publish(sample_msg).await?;

///     subscriber.abort();

///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct Node {
    /// Zenoh session
    session: Session,
}

impl Node {
    /// Creates a zenoh node with a given config
    pub async fn new(config: zenoh::config::Config) -> Result<Self, NodeError> {
        Ok(Self {
            session: Session::new(config).await?,
        })
    }

    /// Creates a new zenoh publisher publishing a prost message
    pub async fn new_publisher<T: prost::Message>(
        &self,
        topic: &str,
    ) -> Result<Publisher<T>, NodeError> {
        let publisher = Publisher::new(topic, self.session.clone()).await?;
        Ok(publisher)
    }

    /// Creates a new zenoh subscriber
    pub async fn new_subscriber<S>(&self, inner: S) -> Result<Subscriber<S>, NodeError> {
        let subscriber = Subscriber::new(self.session.clone(), inner);
        Ok(subscriber)
    }

    /// Subscribes to a prost message
    pub async fn subscribe<T: prost::Message>(
        &self,
        topic: &str,
        subscriber: &mut dyn SubscriberImpl<T>,
    ) -> Result<(), NodeError> {
        let _ = subscriber.subscribe(topic).await?;
        Ok(())
    }
}
