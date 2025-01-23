use prost::Message;
use snafu::Error;
use zenoh_node::builder::NodeBuilder;
use zenoh_node::node::{Abort, Subscribe, SubscriberError};

use async_trait::async_trait;
use std::thread::sleep;
use std::time::Duration;

/// Sample prost protobuf message, in practice this will be generated from
/// a .proto file
#[derive(Clone, PartialEq, Message)]
pub struct SampleMessage {
    #[prost(string, tag = "1")]
    pub name: String,
}

/// My minimal subscriber
pub struct MySubscriber {}

/// Implement Subscribe trait for the SampleMessage
#[async_trait]
impl Subscribe<SampleMessage> for MySubscriber {
    async fn on_data(&mut self, msg: SampleMessage) -> Result<(), SubscriberError> {
        println!("Sample message received with name: {}", msg.name);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut builder = NodeBuilder::new();
    let node = builder.build().await?;

    let my_subscriber = MySubscriber {};
    let mut subscriber = node.new_subscriber(my_subscriber).await?;

    node.subscribe::<SampleMessage>("my_topic", &mut subscriber)
        .await?;

    sleep(Duration::from_millis(50)); // wait until subscriber is up and running

    let publisher = node.new_publisher::<SampleMessage>("my_topic").await?;

    let sample_msg = SampleMessage {
        name: String::from("John Doe"),
    };

    publisher.publish(sample_msg).await?;

    // Normally call join here but minimal example we wont
    // subscriber.join().await?;

    sleep(Duration::from_millis(50)); // give time for subscriber to get message before abort

    subscriber.abort();
    Ok(())
}
