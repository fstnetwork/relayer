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
use futures::{sync::mpsc, Async, Future, Poll, Stream};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;
use tokio_signal::unix::{Signal, SIGHUP, SIGINT, SIGTERM};

use collation::FstRequestConverter;
use ethereum::monitor::{Params as EthereumMonitorParams, Service as EthereumMonitor};
use ethereum::service::{Params as EthereumServiceParams, Service as EthereumService};
use machine::{MachineService, MachineServiceParams, RelayerMode};
use network::{NetworkParams, NetworkService};
use pool::{
    ListAddressFilter, ListAddressFilterMode, PoolParams, PoolService, RequestVerifier,
    TokenSelector, VerifiedRequest,
};
use pricer::{Error as PriceServiceError, PriceService, PriceServiceMode};

use super::rpc_apis;

pub mod config;
mod error;
mod exit_reason;

pub use self::config::Configuration;
pub use self::error::{Error, ErrorKind};
pub use self::exit_reason::ExitReason;

use super::jsonrpc::{
    HttpConfiguration as JsonRpcHttpConfig,
    IpcConfiguration as JsonRpcIpcConfig,
    Service as JsonRpcService,
    ServiceParams as JsonRpcServiceParams,
    // WebSocketConfiguration as JsonRpcWebSocketConfig,
};

#[derive(Clone)]
pub struct ExitHandle {
    sender: mpsc::UnboundedSender<()>,
}

impl traits::ExitHandle for ExitHandle {
    fn shutdown(&mut self) {
        self.sender
            .unbounded_send(())
            .expect("sending exit signal is always successful; qed");
    }
}

pub struct Service {
    ethereum_service: Arc<Mutex<EthereumService>>,
    ethereum_monitor_service: Arc<Mutex<EthereumMonitor<EthereumService>>>,
    price_service: Arc<Mutex<PriceService>>,
    pool_service: Arc<
        Mutex<
            PoolService<
                EthereumService,
                NetworkService,
                ListAddressFilter,
                VerifiedRequest,
                TokenSelector,
                RequestVerifier<EthereumService>,
            >,
        >,
    >,
    network_service: Arc<Mutex<NetworkService>>,
    machine_service: Arc<
        Mutex<
            MachineService<
                EthereumService,
                EthereumMonitor<EthereumService>,
                PoolService<
                    EthereumService,
                    NetworkService,
                    ListAddressFilter,
                    VerifiedRequest,
                    TokenSelector,
                    RequestVerifier<EthereumService>,
                >,
                PriceService,
                FstRequestConverter,
            >,
        >,
    >,

    jsonrpc_service: Box<JsonRpcService>,
    exit_handle: Arc<Mutex<ExitHandle>>,
    exit_handler: Box<Future<Item = ExitReason, Error = ()> + Send>,
}

impl Service {
    pub fn new(config: Configuration) -> Result<Service, Error> {
        let (exit_handler, exit_handle) = {
            let (sender, mut receiver) = mpsc::unbounded::<()>();
            let register_signal = |unix_signal: i32| {
                Box::new(
                    Signal::new(unix_signal)
                        .flatten_stream()
                        .into_future()
                        .then(move |_| Ok(ExitReason::Signal(unix_signal))),
                )
            };

            let exit_signals: Vec<Box<Future<Item = ExitReason, Error = ()> + Send>> = vec![
                register_signal(SIGTERM),
                register_signal(SIGINT),
                register_signal(SIGHUP),
                Box::new(receiver.into_future().then(|_| Ok(ExitReason::Internal))),
            ];

            (
                Box::new(
                    futures::select_all(exit_signals)
                        .and_then(|vec| Ok(vec.0))
                        .map_err(|_| ()),
                ),
                Arc::new(Mutex::new(ExitHandle { sender })),
            )
        };

        let ethereum_service = {
            let params = config.ethereum_params();
            info!(target: "system",
                "Start Ethereum Service with nodes: {:?}",
                params.ethereum_nodes
            );
            Arc::new(Mutex::new(EthereumService::new(params)?))
        };

        let ethereum_monitor_service = {
            let params = config.ethereum_monitor_params();
            info!(target: "system", "Start Ethereum Monitor Service");
            Arc::new(Mutex::new(EthereumMonitor::new(
                ethereum_service.clone(),
                params,
            )))
        };

        let network_service = {
            let params = NetworkParams {};
            info!(target: "system", "Start Network Service");
            Arc::new(Mutex::new(NetworkService::new(params)))
        };

        let pool_service = {
            let params = config.pool_params();
            info!(
                target: "system",
                "Start Request Pool Service, supported tokens: {:?}",
                params.allow_tokens
            );

            let mut token_filter = params.allow_tokens.into_iter().fold(
                ListAddressFilter::new(ListAddressFilterMode::Whitelist),
                |mut token_filter, token_address| {
                    token_filter.add_token(token_address);
                    token_filter
                },
            );

            let request_verifier =
                Arc::new(Mutex::new(RequestVerifier::new(ethereum_service.clone())));
            let request_selector = Arc::new(TokenSelector::new());
            let interval = Duration::from_secs(1);
            Arc::new(Mutex::new(PoolService::new(
                PoolParams {
                    max_count: params.max_count,
                    max_per_sender: params.max_per_sender,
                    max_mem_usage: params.max_mem_usage,
                },
                interval,
                ethereum_service.clone(),
                network_service.clone(),
                token_filter,
                request_verifier,
                request_selector,
            )))
        };

        let price_service = {
            let params = config.pricer_params();
            info!(target: "system", "Start Price Service with mode: {:?}", params);
            Arc::new(Mutex::new(PriceService::new(params)))
        };

        let machine_service = {
            // let relayer_mode = RelayerMode::NotBroadcastTransaction;
            let relayer_mode = RelayerMode::BroadcastTransaction;
            let params = config.machine_params()?;

            let mut machine = MachineService::new(
                relayer_mode,
                params,
                ethereum_service.clone(),
                ethereum_monitor_service.clone(),
                pool_service.clone(),
                price_service.clone(),
            );

            {
                use traits::MachineService;
                let relayer_count = machine.relayer_count();
                match relayer_count {
                    0 => {
                        info!(target: "system",
                            "Inital Relayer Machiner Service with no relayer");
                    }
                    1 => {
                        info!(target: "system",
                            "Initial Relayer Machine Service with {} relayer: {:?}",
                            relayer_count,
                            machine.relayers(),
                        );
                    }
                    _ => {
                        info!(target: "system",
                            "Initial Relayer Machine Service with {} relayers: {:?}",
                            relayer_count,
                            machine.relayers(),
                        );
                    }
                }

                match (config.enable_machine(), relayer_count > 0) {
                    (true, false) => {
                        warn!(target: "system",
                            "Relayer Machine Service is enabled, start Relayer Machine Service but there is no available relayer");
                        machine.start();
                    }
                    (true, true) => {
                        info!(target: "system", "Relayer Machine Service is enabled, start Relayer Machine Service");
                        machine.start();
                    }
                    (false, _) => {
                        info!(target: "system", "Relayer Machine Service is disabled");
                    }
                }
            }

            Arc::new(Mutex::new(machine))
        };

        let jsonrpc_service = {
            let params = config.jsonrpc_params();
            info!(target: "system", "Start JSON-RPC service");

            Box::new(JsonRpcService::new(
                params,
                exit_handle.clone(),
                ethereum_service.clone(),
                network_service.clone(),
                pool_service.clone(),
                machine_service.clone(),
            ))
        };

        Ok(Service {
            ethereum_service,
            ethereum_monitor_service,

            pool_service,
            price_service,
            network_service,
            machine_service,

            jsonrpc_service,

            exit_handler,
            exit_handle,
        })
    }

    #[allow(unused)]
    pub fn start_relayer(&mut self) {
        use traits::MachineService;
        self.machine_service.lock().start();
    }

    #[allow(unused)]
    pub fn stop_relayer(&mut self) {
        use traits::MachineService;
        self.machine_service.lock().stop();
    }

    #[allow(unused)]
    pub fn exit_handle(&self) -> Arc<Mutex<ExitHandle>> {
        self.exit_handle.clone()
    }

    #[allow(unused)]
    pub fn shutdown(&mut self) {
        use traits::ExitHandle;
        self.exit_handle.lock().shutdown();
    }
}

impl Stream for Service {
    type Item = ExitReason;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            match self.exit_handler.poll() {
                Ok(Async::Ready(exit)) => {
                    // TODO gracefully exit
                    self.jsonrpc_service.shutdown();
                    return Ok(Async::Ready(Some(exit)));
                }
                _ => {}
            }

            // FIXME polling ethereum service here
            // self.ethereum_service.

            match self.ethereum_monitor_service.lock().poll() {
                Err(_err) => return Err(()),
                _ => {}
            }

            match self.machine_service.lock().poll() {
                Err(_err) => return Err(()),
                _ => {}
            }

            match self.pool_service.lock().poll() {
                Err(_err) => return Err(()),
                _ => {}
            }

            match self.price_service.lock().poll() {
                Err(_err) => return Err(()),
                _ => {}
            }

            match self.network_service.lock().poll() {
                Err(_err) => return Err(()),
                _ => {}
            }

            match self.jsonrpc_service.poll() {
                Err(_err) => return Err(()),
                _ => {}
            }

            return Ok(Async::NotReady);
        }
    }
}
