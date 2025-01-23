use snafu::Snafu;
use std::error::Error as StdError;
use std::fmt::Debug;
use std::net::AddrParseError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum NodeError {
    /// Error while putting value to zenoh
    SendError {
        source: zenoh::Error,
    },

    /// Error on prost serialization
    #[snafu(display("Unable to serialize structure: {}", source))]
    UnableToSerializeError {
        source: Box<dyn StdError + Send + Sync>,
    },

    /// All senders were dropped and no messages are waiting in the channel, so no further messages can be received.
    ReceiveError,

    /// Cannot create a receiver for the given key
    DeclareReceiverError {
        source: zenoh::Error,
    },

    /// Cannot create a publisher for the given key
    DeclarePublisherError {
        source: zenoh::Error,
    },

    /// Error on prost deserialization
    #[snafu(display("Cannot deserialize message payload on topic: {topic}"))]
    DeserializeError {
        source: Box<dyn StdError + Send + Sync>,
        topic: String,
    },

    EncodeError,

    /// Error publishing the message to topic
    PublishError,

    /// Error subscribing to topic
    SubscribeError,

    /// Error loading the zenoh config file
    LoadConfigError {
        source: zenoh::Error,
    },

    /// Error initializing the session with the given config
    InitializeSession {
        source: zenoh::Error,
    },

    /// Scouting config parse error
    ScoutingConfigError {
        source: AddrParseError,
    },
}
