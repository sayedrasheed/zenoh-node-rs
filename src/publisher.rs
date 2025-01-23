use snafu::ResultExt;
use zenoh::prelude::r#async::AsyncResolve;

use super::error::*;
use crate::session::Session;

use std::{fmt::Debug, marker::PhantomData, sync::Arc};

/// Publisher that publishes protobuf message using a zenoh session
#[derive(Debug, Clone)]
pub struct Publisher<T>
where
    T: prost::Message + Debug,
{
    inner: Arc<PublisherInner<T>>,
    _topic: Arc<str>,
}
impl<T> Publisher<T>
where
    T: prost::Message + Debug,
{
    /// Creates new publisher that publishes on given topic using provided zenoh session
    pub(super) async fn new(topic: &str, session: Session) -> Result<Publisher<T>, NodeError> {
        let inner = PublisherInner::<T>::new(topic, session).await?;
        let publisher = Self {
            inner: Arc::new(inner),
            _topic: Arc::from(topic),
        };

        Ok(publisher)
    }

    /// Publishes protobuf message
    pub async fn publish(&self, msg: T) -> Result<(), NodeError> {
        // publish the data
        self.inner.publish(msg).await?;
        Ok(())
    }
}

/// Publisher helper
#[derive(Debug, Clone)]
struct PublisherInner<T>
where
    T: prost::Message + Debug,
{
    session: Arc<zenoh::Session>,
    topic: Arc<str>,
    _phantom: PhantomData<T>,
}

impl<T> PublisherInner<T>
where
    T: prost::Message + Debug,
{
    /// Create new publisher helper
    async fn new(topic: &str, session: Session) -> Result<PublisherInner<T>, NodeError> {
        let session = session.into_inner();
        //let _key_expr = session.declare_keyexpr(topic).res().await.context(DeclarePublisherSnafu)?;

        Ok(Self {
            session,
            topic: Arc::from(topic),
            _phantom: PhantomData::default(),
        })
    }

    /// Publishes protobuf message
    async fn publish(&self, data: T) -> Result<(), NodeError> {
        let mut buf = Vec::new();
        buf.reserve(data.encoded_len());
        let res = data.encode(&mut buf);

        match res {
            Ok(_) => {
                let topic_str = self.topic.as_ref();
                self.session
                    .put(topic_str, buf)
                    .res()
                    .await
                    .context(DeclarePublisherSnafu)?;
            }
            Err(_) => {
                return Err(NodeError::EncodeError);
            }
        }

        Ok(())
    }
}
