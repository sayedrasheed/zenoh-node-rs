use crate::error::{LoadConfigSnafu, NodeError, ScoutingConfigSnafu};
use crate::node::Node;
use snafu::ResultExt;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// Node Builder
pub struct NodeBuilder {
    /// Optional IPv4 string and port for network selection
    network: Option<(String, u16)>,

    /// Optional path to zenoh json config
    config_path: Option<String>,
}

impl NodeBuilder {
    /// Creates new node builder
    pub fn new() -> Self {
        Self {
            network: None,
            config_path: None,
        }
    }
    /// Builds a zenoh node
    pub async fn build(&mut self) -> Result<Node, NodeError> {
        let mut config = zenoh::config::Config::default();
        if let Some(path) = &self.config_path {
            config = zenoh::config::Config::from_file(path).context(LoadConfigSnafu)?;
        }

        if let Some(mc) = &mut self.network {
            let ipv4 = mc.0.parse::<Ipv4Addr>().context(ScoutingConfigSnafu)?;
            let socket_addr = SocketAddr::new(IpAddr::V4(ipv4), mc.1);
            let _ = config.scouting.multicast.set_address(Some(socket_addr));
        }

        let node = Node::new(config).await?;
        Ok(node)
    }

    /// Setter for zenoh config path
    pub fn set_config_path(&mut self, path: &str) {
        self.config_path = Some(path.to_string());
    }

    // Setter for zenoh network
    pub fn set_network(&mut self, nw: (String, u16)) {
        self.network = Some(nw);
    }
}
