use std::collections::HashMap;
use std::{error, fmt};

use serde_json;

use rand::{seq, thread_rng};
use reqwest;

use common::*;

#[derive(Debug)]
pub enum BlockadeError {
    HttpError(reqwest::Error),
    ServerError(String),
    OtherError(String),
    JsonError(serde_json::Error),
}

impl fmt::Display for BlockadeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BlockadeError::HttpError(ref n) => write!(f, "HTTP error: {:?}", n),
            BlockadeError::OtherError(ref n) => write!(f, "Other error: {:?}", n),
            BlockadeError::ServerError(ref n) => write!(f, "Server error: {:?}", n),
            BlockadeError::JsonError(ref n) => write!(f, "JSON parsing error: {:?}", n),
        }
    }
}

impl From<reqwest::Error> for BlockadeError {
    fn from(error: reqwest::Error) -> BlockadeError {
        return BlockadeError::HttpError(error);
    }
}

impl From<serde_json::Error> for BlockadeError {
    fn from(error: serde_json::Error) -> BlockadeError {
        return BlockadeError::JsonError(error);
    }
}

impl error::Error for BlockadeError {
    fn description(&self) -> &str {
        "Something went wrong with the blockade"
    }
    fn cause(&self) -> Option<&error::Error> {
        return None;
    }
}

#[derive(Debug)]
pub struct BlockadeHandler {
    pub client: reqwest::Client,
    pub host: String,
    pub blockades: Vec<String>,
    pub state: HashMap<String, BlockadeState>,
    pub config: HashMap<String, BlockadeConfig>,
}

impl BlockadeHandler {
    /// Make a new BlockadeHandler that uses a blockade instance
    /// started at "host".
    pub fn new(host: &str) -> Self {
        let client = reqwest::Client::new();
        let mut handler = BlockadeHandler {
            client: client,
            host: host.to_owned(),
            blockades: Vec::new(),
            state: HashMap::new(),
            config: HashMap::new(),
        };
        match handler.execute_list_blockades() {
            Ok(_val) => {
                for i in 0..handler.blockades.len() {
                    let blockade_name = handler.blockades[i].to_owned();
                    match handler.execute_get_blockade(&blockade_name) {
                        Ok(_val) => {}
                        Err(_e) => {}
                    }
                }
            }
            Err(_e) => {}
        }
        return handler;
    }

    /// Returns all container names in default String order (lexicographical).
    pub fn get_all_containers(&mut self, name: &str) -> Result<Vec<String>, BlockadeError> {
        self.execute_get_blockade(name)?;
        let mut all_containers: Vec<String> = if self.state.contains_key(name) {
            self.state[name]
                .containers
                .keys()
                .map(|val: &String| val.clone())
                .collect()
        } else {
            Vec::new()
        };
        all_containers.sort();
        return Ok(all_containers);
    }

    pub fn choose_random_container(&mut self, name: &str) -> Result<String, BlockadeError> {
        if self.state.contains_key(name) && self.state[name].containers.keys().len() >= 1 {
            let mut rng = thread_rng();
            let state = self.state.clone();
            let keys = state.get(name).unwrap().containers.keys();
            let container = seq::sample_iter(&mut rng, keys, 1)
                .unwrap()
                .pop()
                .unwrap()
                .clone();
            return Ok(container.into());
        } else if !self.state.contains_key(name) {
            return Err(BlockadeError::OtherError(String::from(
                "Blockade not found in map",
            )));
        } else {
            return Err(BlockadeError::OtherError(String::from(
                "No containers to choose from",
            )));
        }
    }

    /// Start a blockade from a given name and config struct.
    pub fn start_blockade(
        &mut self,
        name: &str,
        config: BlockadeConfig,
        restart: bool,
    ) -> Result<(), BlockadeError> {
        match self.execute_setup(name, config.clone()) {
            Ok(_) => {}
            Err(e) => {
                if restart {
                    match e {
                        BlockadeError::ServerError(s) => {
                            if s == String::from("Blockade name already exists") {
                                self.destroy_blockade(name)?;
                                self.execute_setup(name, config.clone())?;
                                return Ok(());
                            }
                        }
                        _ => {}
                    }
                }
            }
        };
        self.execute_get_blockade(name)?;
        return Ok(());
    }

    pub fn start_container(&mut self, name: &str, container: &str) -> Result<(), BlockadeError> {
        self.execute_command(name, BlockadeCommand::Start, vec![container.into()])?;
        self.execute_get_blockade(name)?;
        return Ok(());
    }

    /// Stop a container by blockade name and container name.
    pub fn stop_container(&mut self, name: &str, container: &str) -> Result<(), BlockadeError> {
        self.execute_command(name, BlockadeCommand::Stop, vec![container.into()])?;
        self.execute_get_blockade(name)?;
        return Ok(());
    }

    /// Restart a container by blockade name and container name.
    pub fn restart_container(&mut self, name: &str, container: &str) -> Result<(), BlockadeError> {
        self.execute_command(name, BlockadeCommand::Restart, vec![container.into()])?;
        self.execute_get_blockade(name)?;
        return Ok(());
    }

    /// Restart a random-ish container.  Returns the name of the restarted container.
    pub fn restart_one(&mut self, name: &str) -> Result<String, BlockadeError> {
        let container = self.choose_random_container(name)?;
        self.restart_container(name, &container)?;
        return Ok(container);
    }

    /// Kills a container by blockade name and container name.
    pub fn kill_container(&mut self, name: &str, container: &str) -> Result<(), BlockadeError> {
        self.execute_command(name, BlockadeCommand::Kill, vec![container.into()])?;
        self.execute_get_blockade(name)?;
        return Ok(());
    }

    /// Kill a random-ish container.  Returns the name of the killed container.
    pub fn kill_one(&mut self, name: &str) -> Result<String, BlockadeError> {
        let container = self.choose_random_container(name)?;
        self.kill_container(name, &container)?;
        return Ok(container);
    }

    /// Makes partitions according to the given nested Vec<Vec<String>> of container names.
    pub fn make_partitions(
        &mut self,
        name: &str,
        partitions: Vec<Vec<String>>,
    ) -> Result<(), BlockadeError> {
        self.execute_partition(name, partitions)?;
        self.execute_get_blockade(name)?;
        return Ok(());
    }

    /// Puts all containers in one partition and restores the network QoS.
    pub fn heal_partitions(&mut self, name: &str) -> Result<(), BlockadeError> {
        self.execute_restore_network(name)?;
        self.execute_get_blockade(name)?;
        return Ok(());
    }

    /// Makes the network condition generally bad.  Introduces at least latency and dropped packets
    /// potentially also causes reordering of some magnitude.
    pub fn make_net_unreliable(&mut self, name: &str) -> Result<(), BlockadeError> {
        let all_containers = self.get_all_containers(name)?;
        self.execute_net_command(name, BlockadeNetStatus::Flaky, all_containers)?;
        self.execute_get_blockade(name)?;
        return Ok(());
    }

    /// Makes the network condition as good as can be given the host conditions.  Generally this
    /// means near perfect since the containers are usually on the local machine and the OS is
    /// reasonably good about pushing packets.
    pub fn make_net_fast(&mut self, name: &str) -> Result<(), BlockadeError> {
        let all_containers = self.get_all_containers(name)?;
        self.execute_net_command(name, BlockadeNetStatus::Fast, all_containers)?;
        self.execute_get_blockade(name)?;
        return Ok(());
    }

    /// Shuts down the blockade and all of its containers.  Probably don't want to use this
    /// blockade afterward, considering it's pretty final.
    pub fn destroy_blockade(&mut self, name: &str) -> Result<(), BlockadeError> {
        self.execute_get_blockade(name)?;
        self.execute_delete_blockade(name)?;
        return Ok(());
    }

    pub fn fetch_state(&mut self) -> Result<(), BlockadeError> {
        self.execute_list_blockades()?;
        let blockades = self.blockades.clone();
        for blockade in blockades.iter() {
            self.execute_get_blockade(&blockade)?;
        }
        return Ok(());
    }

    fn execute_setup(&mut self, name: &str, config: BlockadeConfig) -> Result<(), BlockadeError> {
        self.config.insert(name.into(), config.clone());

        let json = serde_json::to_string_pretty(&config).expect("Failed to serialize config");
        trace!("Config: {}", json);

        let mut res = self.client
            .post(format!("{}/blockade/{}", self.host, name).as_str())
            .json(&config)
            .send()?;

        debug!("Posted to server with status: {}", res.status());

        if res.status().is_success() {
            return Ok(());
        } else {
            return Err(BlockadeError::ServerError(res.text()?));
        }
    }

    fn execute_command(
        &mut self,
        name: &str,
        command: BlockadeCommand,
        containers: Vec<String>,
    ) -> Result<(), BlockadeError> {
        let args = BlockadeCommandArgs {
            command,
            container_names: containers,
        };

        let mut res = self.client
            .post(format!("{}/blockade/{}/action", self.host, name).as_str())
            .json(&args)
            .send()?;

        debug!("Posted to server with status: {}", res.status());

        if res.status().is_success() {
            return Ok(());
        } else {
            return Err(BlockadeError::ServerError(res.text()?));
        }
    }

    fn execute_net_command(
        &mut self,
        name: &str,
        network_state: BlockadeNetStatus,
        container_names: Vec<String>,
    ) -> Result<(), BlockadeError> {
        let args = BlockadeNetArgs {
            network_state,
            container_names: container_names,
        };

        let mut res = self.client
            .post(format!("{}/blockade/{}/network_state", self.host, name).as_str())
            .json(&args)
            .send()?;

        debug!("Posted to server with status: {}", res.status());

        if res.status().is_success() {
            return Ok(());
        } else {
            return Err(BlockadeError::ServerError(res.text()?));
        }
    }

    fn execute_partition(
        &mut self,
        name: &str,
        partitions: Vec<Vec<String>>,
    ) -> Result<(), BlockadeError> {
        let args = BlockadePartitionArgs { partitions };

        let mut res = self.client
            .post(format!("{}/blockade/{}/partitions", self.host, name).as_str())
            .json(&args)
            .send()?;

        debug!("Posted to server with status: {}", res.status());

        if res.status().is_success() {
            return Ok(());
        } else {
            return Err(BlockadeError::ServerError(res.text()?));
        }
    }

    fn execute_restore_network(&mut self, name: &str) -> Result<(), BlockadeError> {
        let mut res = self.client
            .delete(format!("{}/blockade/{}/partitions", self.host, name).as_str())
            .send()?;

        debug!("Sent delete to server with status: {}", res.status());

        if res.status().is_success() {
            return Ok(());
        } else {
            return Err(BlockadeError::ServerError(res.text()?));
        }
    }

    fn execute_list_blockades(&mut self) -> Result<(), BlockadeError> {
        let mut res = self.client
            .get(format!("{}/blockade", self.host).as_str())
            .send()?;

        debug!("Sent get to server with status: {}", res.status());

        if res.status().is_success() {
            let raw_text = res.text()?;
            debug!("Raw response from server: {:#?}", &raw_text);
            let v: HashMap<String, Vec<String>> = serde_json::from_str(&raw_text)?;
            self.blockades = match v.get("blockades") {
                Some(n) => (n.clone()).into(),
                None => Vec::new(),
            };
            return Ok(());
        } else {
            return Err(BlockadeError::ServerError(res.text()?));
        }
    }

    fn execute_get_blockade(&mut self, name: &str) -> Result<(), BlockadeError> {
        let mut res = self.client
            .get(format!("{}/blockade/{}", self.host, name).as_str())
            .send()?;

        debug!("Sent get to server with status: {}", res.status());

        if res.status().is_success() {
            let raw_text = res.text()?;
            debug!("Raw response from server: {:#?}", &raw_text);
            let s: BlockadeState = serde_json::from_str(&raw_text)?;
            self.state.insert(name.into(), s);
            return Ok(());
        } else {
            return Err(BlockadeError::ServerError(res.text()?));
        }
    }

    fn execute_delete_blockade(&mut self, name: &str) -> Result<(), BlockadeError> {
        let mut res = self.client
            .delete(format!("{}/blockade/{}", self.host, name).as_str())
            .send()?;

        debug!("Sent delete to server with status: {}", res.status());

        if res.status().is_success() {
            if self.state.contains_key(name) {
                self.state.remove(name);
            }
            return Ok(());
        } else {
            return Err(BlockadeError::ServerError(res.text()?));
        }
    }
}
