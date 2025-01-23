use std::sync::Arc;

use snafu::ResultExt;
use zenoh::prelude::r#async::AsyncResolve;

use crate::error::{InitializeSessionSnafu, NodeError};

/// Zenoh session wrapper
#[derive(Clone, Debug)]
pub struct Session(Arc<zenoh::Session>);

impl Session {
    /// Into zenoh session
    pub(crate) fn into_inner(self) -> Arc<zenoh::Session> {
        self.0
    }

    // Create default zenoh session
    pub async fn new_with_default() -> Result<Session, NodeError> {
        let config = zenoh::config::Config::default();
        Self::new(config).await
    }

    /// Create new zenoh session with given zenoh config
    pub async fn new(config: zenoh::config::Config) -> Result<Session, NodeError> {
        let session = zenoh::open(config.clone())
            .res()
            .await
            .context(InitializeSessionSnafu)?;
        Ok(Self::from(session.into_arc()))
    }

    /// From zenoh session
    pub fn from(session: Arc<zenoh::Session>) -> Self {
        Self(session)
    }
}
