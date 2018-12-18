// Copyright 2017-2018 FST Network Pte. Ltd.
// This file is part of FST Relayer.

// FST Relayer is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// FST Relayer is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with FST Relayer. If not, see <http://www.gnu.org/licenses/>.
use ethereum_types::{Address, U256};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use ethkey::{KeyPair, Password};
use ethstore::Crypto;

use collation::{FstRequestConverter, RequestDispatcher};

use super::{
    EthereumMonitorParams, EthereumServiceParams, JsonRpcHttpConfig, JsonRpcIpcConfig,
    JsonRpcServiceParams, MachineServiceParams, PriceServiceMode,
};

use super::rpc_apis;

mod error;
pub use self::error::{Error, ErrorKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumService {
    pub ethereum_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthereumMonitor {
    pub interval_millis: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Machine {
    pub disable: bool,
    pub dispatcher: Address,
    pub chain_id: Option<u64>,
    pub interval_secs: u64,
    pub confirmation_count: u32,
    pub relayers: HashMap<Address, Relayer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relayer {
    pub keyfile: String,
    pub password_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PricerMode {
    #[serde(rename = "fixed")]
    Fixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pricer {
    pub mode: PricerMode,
    pub fixed_gas_price_in_gwei: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    // port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pool {
    pub max_count: usize,
    pub max_per_sender: usize,
    pub max_mem_usage: usize,

    pub interval_secs: u64,

    pub allow_tokens: Vec<Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpc {
    pub http: Option<JsonRpcHttp>,
    pub ipc: Option<JsonRpcIpc>,
    // pub websocket: Option<JsonRpcWebSocket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcIpc {
    pub disable: bool,
    pub apis: Vec<String>,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcHttp {
    pub disable: bool,
    pub apis: Vec<String>,
    pub interface: String,
    pub port: u16,
    pub thread_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcWebSocket {
    pub disable: bool,
    pub apis: Vec<String>,
    pub interface: String,
    pub port: u16,
    pub thread_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    /// Ethereum Service
    pub ethereum: EthereumService,

    /// Ethereum Monitor Service
    pub ethereum_monitor: EthereumMonitor,

    /// Request Pool Service
    pub pool: Pool,

    /// Relayer Service
    #[serde(rename = "relayer")]
    pub machine: Machine,

    /// Price Service
    pub pricer: Pricer,

    //// Network Service
    // pub network: Network,
    /// JSON-RPC Service
    pub jsonrpc: JsonRpc,
}

impl Configuration {
    pub fn recover_keypair(
        keyfile_path: &PathBuf,
        password_file: &PathBuf,
    ) -> Result<KeyPair, error::Error> {
        fn read_file(file_path: &PathBuf) -> Result<String, error::Error> {
            let mut file_content = String::new();
            File::open(file_path)?.read_to_string(&mut file_content)?;

            Ok(file_content)
        }

        let secret = {
            let crypto = {
                let mut key_file_content = read_file(keyfile_path)?;
                let value: serde_json::Value = serde_json::from_str(&key_file_content)?;
                Crypto::from_str(&value["crypto"].to_string())?
            };

            let password = Password::from(read_file(password_file)?.trim().to_owned());
            match crypto.secret(&password) {
                Ok(secret) => secret,
                Err(err) => return Err(Error::from(ErrorKind::EthStore(err))),
            }
        };

        Ok(KeyPair::from_secret(secret)?)
    }

    pub fn interface(interface: &String) -> IpAddr {
        match interface.as_str() {
            "all" => IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            "local" | "localhost" | _ => IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        }
    }

    #[allow(unused)]
    pub fn enable_machine(&self) -> bool {
        !self.machine.disable
    }

    #[allow(unused)]
    pub fn disable_machine(&self) -> bool {
        self.machine.disable
    }

    pub fn ethereum_params(&self) -> EthereumServiceParams {
        EthereumServiceParams {
            ethereum_nodes: self.ethereum.ethereum_nodes.clone(),
        }
    }

    pub fn ethereum_monitor_params(&self) -> EthereumMonitorParams {
        EthereumMonitorParams {
            ticker_interval: Duration::from_millis(self.ethereum_monitor.interval_millis),
        }
    }

    pub fn machine_params(
        &self,
    ) -> Result<MachineServiceParams<FstRequestConverter>, error::Error> {
        let config = self.machine.clone();

        if config.interval_secs == 0 {
            return Err(Error::from(ErrorKind::InvalidRelayInterval));
        }

        let mut relayer_keypairs: Vec<KeyPair> = Vec::new();
        for (address, r) in config.relayers.into_iter() {
            let keyfile = match resolve_path(&r.keyfile) {
                Some(path) => path,
                None => return Err(Error::from(ErrorKind::ResolveFilePathFailed(r.keyfile))),
            };

            let password_file = match resolve_path(&r.password_file) {
                Some(path) => path,
                None => {
                    return Err(Error::from(ErrorKind::ResolveFilePathFailed(
                        r.password_file,
                    )))
                }
            };

            match Self::recover_keypair(&keyfile, &password_file) {
                Ok(keypair) => {
                    info!(target: "system",
                        "Relayer address {:?} is recovered from {}",
                        keypair.address(),
                        r.keyfile
                    );
                    relayer_keypairs.push(keypair);
                }
                Err(err) => {
                    return Err(Error::from(ErrorKind::RecoverPrivateKeyFailed(
                        format!("{:?}", err),
                        address,
                        r.keyfile,
                        r.password_file,
                    )))
                }
            }
        }

        Ok(MachineServiceParams {
            dispatcher: RequestDispatcher::new(config.dispatcher, FstRequestConverter::new()),
            chain_id: config.chain_id,
            interval: Duration::from_secs(config.interval_secs),
            relayer_keypairs,
            confirmation_count: config.confirmation_count,
        })
    }

    pub fn pricer_params(&self) -> PriceServiceMode {
        match self.pricer.mode {
            PricerMode::Fixed => PriceServiceMode::Fixed {
                gas_price: U256::from(self.pricer.fixed_gas_price_in_gwei.unwrap_or(5))
                    * U256::from(1_000_000_000),
                token_prices: HashMap::new(),
            },
            // _ => PriceServiceMode::Fixed {
            //     gas_price: U256::from(5) * U256::from(1_000_000_000),
            //     token_prices: HashMap::new(),
            // },
        }
    }

    pub fn jsonrpc_params(&self) -> JsonRpcServiceParams {
        let config = self.jsonrpc.clone();
        let http_config = config.http.map(|http| JsonRpcHttpConfig {
            disable: http.disable,
            apis: rpc_apis::ApiSet::from_strings(http.apis),
            socket_address: SocketAddr::new(Self::interface(&http.interface), http.port),
            thread_count: http.thread_count,
        });

        let ipc_config = config.ipc.map(|ipc| JsonRpcIpcConfig {
            disable: ipc.disable,
            apis: rpc_apis::ApiSet::from_strings(ipc.apis),
            ipc_path: ipc.path,
        });

        JsonRpcServiceParams {
            http_config,
            ipc_config,
            websocket_config: None,
        }
    }

    pub fn pool_params(&self) -> Pool {
        self.pool.clone()
    }

    pub fn new_example() -> Configuration {
        Configuration {
            ethereum: EthereumService {
                ethereum_nodes: vec!["http://127.0.0.1:8545".to_owned()],
            },
            ethereum_monitor: EthereumMonitor {
                interval_millis: 1000,
            },
            pool: Pool {
                max_count: 10240,
                max_per_sender: 16,
                max_mem_usage: 8 * 1024 * 1024,

                interval_secs: 3,

                allow_tokens: vec![Address::from("3830f7aF866fae79e4f6b277be17593bf96bee3b")],
            },
            machine: Machine {
                disable: false,
                chain_id: None,
                interval_secs: 5,
                confirmation_count: 12,
                dispatcher: Address::from("4ac3b5f5162b12f3f5c81a5db2348405e9462c23"),
                relayers: {
                    let mut relayers = HashMap::new();
                    relayers.insert(
                        Address::from("0000000000000000000000000000000000000000"),
                        Relayer {
                            keyfile: "/tmp/my-keyfile-1.json".to_owned(),
                            password_file: "/tmp/my-passphrase-1".to_owned(),
                        },
                    );

                    relayers.insert(
                        Address::from("0101010101010101010101010101010101010101"),
                        Relayer {
                            keyfile: "/tmp/my-keyfile-2.json".to_owned(),
                            password_file: "/tmp/my-passphrase-2".to_owned(),
                        },
                    );

                    relayers
                },
            },
            pricer: Pricer {
                mode: PricerMode::Fixed,
                fixed_gas_price_in_gwei: Some(5),
            },
            jsonrpc: JsonRpc {
                http: Some(JsonRpcHttp {
                    disable: false,
                    apis: vec!["token".to_owned()],
                    interface: "local".to_owned(),
                    port: 4982,
                    thread_count: 2,
                }),
                ipc: Some(JsonRpcIpc {
                    disable: false,
                    apis: vec!["token".to_owned()],
                    path: "/tmp/fst-relayer.ipc".to_owned(),
                }),
            },
        }
    }
}

impl Default for Configuration {
    fn default() -> Configuration {
        Configuration {
            ethereum: EthereumService {
                ethereum_nodes: vec!["http://127.0.0.1:8545".to_owned()],
            },
            ethereum_monitor: EthereumMonitor {
                interval_millis: 1000,
            },
            pool: Pool {
                max_count: 10240,
                max_per_sender: 16,
                max_mem_usage: 8 * 1024 * 1024,

                interval_secs: 3,

                allow_tokens: Default::default(),
            },
            machine: Machine {
                disable: false,
                dispatcher: Default::default(),
                relayers: Default::default(),
                chain_id: None,
                interval_secs: 3,
                confirmation_count: 12,
            },
            pricer: Pricer {
                mode: PricerMode::Fixed,
                fixed_gas_price_in_gwei: Some(1),
            },
            jsonrpc: JsonRpc {
                http: Some(JsonRpcHttp {
                    disable: false,
                    apis: vec!["token".to_owned()],
                    interface: "local".to_owned(),
                    port: 4982, // 0x1376 == 4982
                    thread_count: 4,
                }),
                ipc: Some(JsonRpcIpc {
                    disable: false,
                    apis: vec!["token".to_owned()],
                    path: "/tmp/fst-relayer.ipc".to_owned(),
                }),
            },
        }
    }
}

fn resolve_path(path_str: &String) -> Option<std::path::PathBuf> {
    use std::path::{Component, PathBuf};

    let path_buf = PathBuf::from(path_str);
    let mut components = path_buf.components();

    let prefix: PathBuf = match components.next() {
        Some(Component::Normal(c)) => match c.to_str()? {
            "$HOME" => PathBuf::from(dirs::home_dir()?),
            "$XDG_CONFIG_HOME" => PathBuf::from(dirs::config_dir()?),
            "$XDG_DATA_HOME" => PathBuf::from(dirs::data_local_dir()?),
            _ => return None,
        },
        Some(c) => PathBuf::from(c.as_os_str()),
        None => {
            return None;
        }
    };

    Some(components.fold(prefix, |mut path_buf, component| {
        path_buf.push(component);
        path_buf
    }))
}

#[cfg(test)]
mod tests {
    // use super::Configuration;
}
