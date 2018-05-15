use std::collections::HashMap;
use std::net::Ipv4Addr;

use serde::{Serialize, Serializer};
use serde_aux::prelude::*;

pub trait Stringify {
    fn to_str(&self) -> &str;
    fn from_str(val: &str) -> Self;
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum BlockadeCommand {
    Start,
    Stop,
    Restart,
    Kill,
}

impl Stringify for BlockadeCommand {
    fn to_str(&self) -> &str {
        return match *self {
            BlockadeCommand::Start => "start",
            BlockadeCommand::Stop => "stop",
            BlockadeCommand::Restart => "restart",
            BlockadeCommand::Kill => "kill",
            //x => panic!("Unexpected enum input {:?}", x)
        };
    }
    fn from_str(val: &str) -> Self {
        return match val {
            "start" => BlockadeCommand::Start,
            "stop" => BlockadeCommand::Stop,
            "restart" => BlockadeCommand::Restart,
            "kill" => BlockadeCommand::Kill,
            x => panic!("Unexpected enum input {:?}", x),
        };
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum BlockadeNetStatus {
    Fast,
    Slow,
    Duplicate,
    Flaky,
    Unknown,
}

impl Stringify for BlockadeNetStatus {
    fn to_str(&self) -> &str {
        return match *self {
            BlockadeNetStatus::Fast => "fast",
            BlockadeNetStatus::Slow => "slow",
            BlockadeNetStatus::Duplicate => "duplicate",
            BlockadeNetStatus::Flaky => "flaky",
            BlockadeNetStatus::Unknown => "unknown",
            //x => panic!("Unexpected enum input {:?}", x)
        };
    }
    fn from_str(val: &str) -> Self {
        return match val {
            "NORMAL" => BlockadeNetStatus::Fast,
            "FAST" => BlockadeNetStatus::Fast,
            "SLOW" => BlockadeNetStatus::Slow,
            "DUPLICATE" => BlockadeNetStatus::Duplicate,
            "FLAKY" => BlockadeNetStatus::Flaky,
            "UNKNOWN" => BlockadeNetStatus::Unknown,
            x => panic!("Unexpected enum input {:?}", x),
        };
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum BlockadeContainerStatus {
    Up,
    Down,
    Missing,
}

impl Stringify for BlockadeContainerStatus {
    fn to_str(&self) -> &str {
        return match *self {
            BlockadeContainerStatus::Up => "up",
            BlockadeContainerStatus::Down => "down",
            BlockadeContainerStatus::Missing => "missing",
            //x => panic!("Unexpected enum input {:?}", x)
        };
    }
    fn from_str(val: &str) -> Self {
        return match val {
            "UP" => BlockadeContainerStatus::Up,
            "DOWN" => BlockadeContainerStatus::Down,
            "MISSING" => BlockadeContainerStatus::Missing,
            x => panic!("Unexpected enum input {:?}", x),
        };
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BlockadeContainer {
    pub image: String,
    pub hostname: String,
    pub volumes: HashMap<String, String>,
    pub expose: Vec<u16>,
    pub ports: HashMap<u16, u16>,
    pub links: HashMap<String, String>,
    pub command: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BlockadeNetConfig {
    pub flaky: String,
    pub slow: String,
    pub driver: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BlockadeConfig {
    pub containers: HashMap<String, BlockadeContainer>,
    pub network: BlockadeNetConfig,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BlockadeCommandArgs {
    #[serde(deserialize_with = "deserialize_struct_case_insensitive")]
    pub command: BlockadeCommand,
    pub container_names: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BlockadeNetArgs {
    #[serde(deserialize_with = "deserialize_struct_case_insensitive")]
    pub network_state: BlockadeNetStatus,
    pub container_names: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BlockadePartitionArgs {
    pub partitions: Vec<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BlockadeState {
    pub containers: HashMap<String, BlockadeContainerState>,
}

fn none_str_resource() -> Option<String> {
    return None;
}

fn none_u32_resource() -> Option<u32> {
    return None;
}

fn ip_default_resource() -> Ipv4Addr {
    return Ipv4Addr::new(0, 0, 0, 0);
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BlockadeContainerState {
    pub container_id: String,
    #[serde(default = "none_str_resource")]
    pub device: Option<String>,
    #[serde(default = "ip_default_resource")]
    pub ip_address: Ipv4Addr,
    pub name: String,

    #[serde(deserialize_with = "deserialize_struct_case_insensitive")]
    pub network_state: BlockadeNetStatus,

    #[serde(default = "none_u32_resource")]
    pub partition: Option<u32>,

    #[serde(deserialize_with = "deserialize_struct_case_insensitive")]
    pub status: BlockadeContainerStatus,
}

impl Default for BlockadeContainer {
    fn default() -> Self {
        return BlockadeContainer {
            image: String::from("rust"),
            hostname: String::from("c0"),
            volumes: HashMap::new(),
            expose: Vec::new(),
            ports: HashMap::new(),
            links: HashMap::new(),
            command: None,
        };
    }
}

impl Default for BlockadeNetConfig {
    fn default() -> Self {
        return BlockadeNetConfig {
            flaky: String::from("10%"),
            slow: String::from("75ms 100ms distribution normal"),
            driver: String::from("udn"),
        };
    }
}

impl Default for BlockadeConfig {
    fn default() -> Self {
        return BlockadeConfig {
            containers: HashMap::new(),
            network: BlockadeNetConfig::default(),
        };
    }
}

impl Default for BlockadeCommandArgs {
    fn default() -> Self {
        return BlockadeCommandArgs {
            command: BlockadeCommand::Start,
            container_names: Vec::new(),
        };
    }
}

impl Default for BlockadeNetArgs {
    fn default() -> Self {
        return BlockadeNetArgs {
            network_state: BlockadeNetStatus::Fast,
            container_names: Vec::new(),
        };
    }
}

impl Default for BlockadePartitionArgs {
    fn default() -> Self {
        return BlockadePartitionArgs {
            partitions: Vec::new(),
        };
    }
}

impl Default for BlockadeState {
    fn default() -> Self {
        return BlockadeState {
            containers: Default::default(),
        };
    }
}

impl Default for BlockadeContainerState {
    fn default() -> Self {
        return BlockadeContainerState {
            container_id: String::new(),
            device: None,
            ip_address: Ipv4Addr::new(127, 0, 0, 2),
            name: String::new(),
            network_state: BlockadeNetStatus::Unknown,
            partition: Some(0),
            status: BlockadeContainerStatus::Missing,
        };
    }
}

macro_rules! serialize_impl {
    ($($t:ty)*) => ($(
        impl Serialize for $t {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer
            {
                serializer.serialize_str(self.to_str())
            }
        }
    )*)
}

serialize_impl!(BlockadeCommand);
serialize_impl!(BlockadeNetStatus);
serialize_impl!(BlockadeContainerStatus);
