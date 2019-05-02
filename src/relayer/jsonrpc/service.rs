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
use futures::{sync::oneshot, Async, Future, Poll, Stream};
use parking_lot::Mutex;
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;

use crate::collation::FstRequestConverter;
use crate::ethereum::monitor::Service as EthereumMonitor;
use crate::ethereum::service::Service as EthereumService;
use crate::machine;
use crate::network::NetworkService;
use crate::pool::{self, ListAddressFilter, RequestVerifier, TokenSelector, VerifiedRequest};
use crate::pricer::PriceService;

use super::relayer_service::ExitHandle;
use super::rpc_apis;
use super::v1::{Admin, Network, Pool, Relayer, SystemInfo, Token};
use super::v1::{AdminApi, NetworkApi, PoolApi, RelayerApi, SystemInfoApi, TokenApi};
use super::{HttpServerBuilder, IpcServerBuilder, JsonRpcIoHandler};

type PoolService = pool::PoolService<
    EthereumService,
    NetworkService,
    ListAddressFilter,
    VerifiedRequest,
    TokenSelector,
    RequestVerifier<EthereumService>,
>;
type MachineService = machine::MachineService<
    EthereumService,
    EthereumMonitor<EthereumService>,
    PoolService,
    PriceService,
    FstRequestConverter,
>;

struct ThreadHandler {
    shutdown_sender: oneshot::Sender<()>,
    thread_handler: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceParams {
    pub ipc_config: Option<IpcConfiguration>,
    pub http_config: Option<HttpConfiguration>,
    pub websocket_config: Option<WebSocketConfiguration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcConfiguration {
    pub disable: bool,
    pub apis: rpc_apis::ApiSet,
    pub ipc_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfiguration {
    pub disable: bool,
    pub apis: rpc_apis::ApiSet,
    pub socket_address: SocketAddr,
    pub thread_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfiguration {
    pub disable: bool,
    pub apis: rpc_apis::ApiSet,
    pub socket_address: SocketAddr,
    pub thread_count: usize,
}

pub struct Service {
    thread_handler: Option<ThreadHandler>,
}

#[derive(Clone)]
struct RelayerService {
    exit_handler: Arc<Mutex<ExitHandle>>,
    ethereum_service: Arc<Mutex<EthereumService>>,
    network_service: Arc<Mutex<NetworkService>>,
    pool_service: Arc<Mutex<PoolService>>,
    machine_service: Arc<Mutex<MachineService>>,
}

impl Service {
    pub fn new(
        params: ServiceParams,
        exit_handler: Arc<Mutex<ExitHandle>>,
        ethereum_service: Arc<Mutex<EthereumService>>,
        network_service: Arc<Mutex<NetworkService>>,
        pool_service: Arc<Mutex<PoolService>>,
        machine_service: Arc<Mutex<MachineService>>,
    ) -> Service {
        let (shutdown_sender, shutdown_receiver) = oneshot::channel::<()>();

        let ServiceParams {
            ipc_config,
            http_config,
            websocket_config: _,
        } = params;

        let relayer_service = RelayerService {
            exit_handler,
            ethereum_service,
            network_service,
            pool_service,
            machine_service,
        };

        let ipc_server = Self::start_ipc(ipc_config, relayer_service.clone());
        let http_server = Self::start_http(http_config, relayer_service);

        let thread_handler = thread::spawn({
            move || {
                // wait for shutdown signal
                shutdown_receiver
                    .map(|_| info!(target: "system", "Stop JSON-RPC service"))
                    .wait()
                    .unwrap();

                http_server.map(|http_server| {
                    info!(target: "system", "Shutdown JSON-RPC HTTP server");
                    http_server.close();
                });

                ipc_server.map(|ipc_server| {
                    info!(target: "system", "Shutdown JSON-RPC IPC server");
                    ipc_server.close();
                });
            }
        });

        Service {
            thread_handler: Some(ThreadHandler {
                thread_handler,
                shutdown_sender,
            }),
        }
    }

    fn new_handler(apis: rpc_apis::ApiSet, relayer_service: RelayerService) -> JsonRpcIoHandler {
        let mut handler = jsonrpc_core::IoHandler::new();
        let RelayerService {
            exit_handler,
            ethereum_service,
            network_service,
            pool_service,
            machine_service,
        } = relayer_service;

        for api in apis.apis().into_iter() {
            use rpc_apis::Api;
            match api {
                Api::Admin => {
                    let admin = Admin::new(exit_handler.clone(), ethereum_service.clone());
                    handler.extend_with(admin.to_delegate());
                }
                Api::SystemInfo => {
                    let system_info = SystemInfo::new()
                        .name(crate_name!().to_owned())
                        .version(crate_version!().to_owned());

                    handler.extend_with(system_info.to_delegate());
                }
                Api::Network => {
                    let network = Network::new(network_service.clone());
                    handler.extend_with(network.to_delegate());
                }
                Api::Pool => {
                    let pool = Pool::new(pool_service.clone());
                    handler.extend_with(pool.to_delegate());
                }
                Api::Relayer => {
                    let relayer = Relayer::new(machine_service.clone());
                    handler.extend_with(relayer.to_delegate());
                }
                Api::Token => {
                    let token = Token::new(ethereum_service.clone(), pool_service.clone());
                    handler.extend_with(token.to_delegate());
                }
            }
        }

        handler
    }

    pub fn shutdown(&mut self) {
        match self.thread_handler.take() {
            Some(ThreadHandler {
                shutdown_sender,
                thread_handler,
            }) => {
                shutdown_sender
                    .send(())
                    .expect("receiver is always existed; qed");
                thread_handler.join().expect("thread is not joined; qed");
            }
            None => {
                warn!("emit shutdown when service is shutting down");
            }
        }
    }

    fn start_ipc(
        ipc_config: Option<IpcConfiguration>,
        relayer_service: RelayerService,
    ) -> Option<jsonrpc_ipc_server::Server> {
        ipc_config.and_then(|ipc_config| {
            let IpcConfiguration {
                disable,
                ipc_path,
                apis,
            } = ipc_config;

            if disable {
                info!(target: "system", "JSON-RPC IPC server is disabled.");
                return None;
            }

            info!(target: "system",
                "Start JSON-RPC IPC server on {} with API {}",
                ipc_path, apis
            );

            IpcServerBuilder::new(Self::new_handler(apis, relayer_service))
                .start(&ipc_path)
                .map_err(|err| {
                    error!(target: "system", "Failed to start JSON-RPC IPC server: {:?}", err);
                })
                .ok()
        })
    }

    fn start_http(
        http_config: Option<HttpConfiguration>,
        relayer_service: RelayerService,
    ) -> Option<jsonrpc_http_server::Server> {
        http_config.and_then(|http_config| {
            let HttpConfiguration {
                disable,
                apis,
                socket_address,
                thread_count,
            } = http_config;

            if disable {
                info!(target: "system", "JSON-RPC HTTP server is disabled.");
                return None;
            }

            info!(target: "system",
                "Start JSON-RPC HTTP server on {} with API: {}",
                socket_address, apis
            );

            HttpServerBuilder::new(Self::new_handler(apis, relayer_service))
                .threads(thread_count)
                .start_http(&socket_address)
                .map_err(|err| {
                    error!(target: "system", "Failed to start JSON-RPC HTTP server: {:?}", err);
                })
                .ok()
        })
    }
}

impl Stream for Service {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            if self.thread_handler.is_some() {
                return Ok(Async::NotReady);
            }
            return Ok(Async::Ready(None));
        }
    }
}
