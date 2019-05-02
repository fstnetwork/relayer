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
use ethereum_types::{Address, H256};
use ethkey::KeyPair;
use futures::{Async, Future, Poll, Stream};
use parking_lot::Mutex;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::timer::Interval;

use crate::collation::{RequestConverter, RequestDispatcher};
use crate::ethereum::monitor::Error as EthereumMonitorError;
use crate::ethereum::service::Error as EthereumServiceError;
use crate::pricer::Error as PriceServiceError;
use crate::traits::{
    self, EthereumMonitor, EthereumService, PoolRequestTag, PoolService, PriceService,
};
use crate::types::SignedRequest;

use super::{
    Error, ErrorKind, RelayerEvent, RelayerInfo, RelayerMachine, RelayerMode, RelayerParams,
    RelayerState,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Status {
    #[serde(rename = "isWorking")]
    is_working: bool,

    #[serde(rename = "relayerCount")]
    relayer_count: usize,

    #[serde(rename = "relayerInfos")]
    relayer_infos: Vec<RelayerInfo>,
}

pub struct Params<C>
where
    C: RequestConverter,
{
    pub relayer_keypairs: Vec<KeyPair>,
    pub dispatcher: RequestDispatcher<C>,
    pub chain_id: Option<u64>,

    pub interval: Duration,
    pub confirmation_count: u32,
}

pub struct Service<E, M, P, G, C>
where
    E: EthereumService<Error = EthereumServiceError>,
    M: EthereumMonitor<MonitorError = EthereumMonitorError>,
    P: PoolService<SignedRequest = SignedRequest, Hash = H256>,
    G: PriceService<PricerError = PriceServiceError>,
    C: RequestConverter,
{
    running: bool,

    ethereum: Arc<Mutex<E>>,
    ethereum_monitor: Arc<Mutex<M>>,
    pool: Arc<Mutex<P>>,
    gas_pricer: Arc<Mutex<G>>,

    relayer_machines: Mutex<Vec<RelayerMachine<E, M, P, G, C>>>,
    retired_relayer_machines: Mutex<HashSet<Address>>,

    ticker: Interval,
}

impl<E, M, P, G, C> Service<E, M, P, G, C>
where
    E: EthereumService<Error = EthereumServiceError>,
    M: EthereumMonitor<MonitorError = EthereumMonitorError>,
    P: PoolService<SignedRequest = SignedRequest, Hash = H256>,
    G: PriceService<PricerError = PriceServiceError>,
    C: RequestConverter,
{
    pub fn new(
        mode: RelayerMode,
        mut params: Params<C>,
        ethereum: Arc<Mutex<E>>,
        ethereum_monitor: Arc<Mutex<M>>,
        pool: Arc<Mutex<P>>,
        gas_pricer: Arc<Mutex<G>>,
    ) -> Service<E, M, P, G, C> {
        // sort and remove duplicated keypairs
        params
            .relayer_keypairs
            .sort_unstable_by(|a, b| a.address().cmp(&b.address()));
        params.relayer_keypairs.dedup();

        // create new relayer machines from keypairs
        let relayer_machines: Mutex<Vec<_>> = Mutex::new(
            params
                .relayer_keypairs
                .iter()
                .map(|keypair| {
                    let machine = RelayerMachine::new(
                        mode,
                        RelayerParams {
                            keypair: keypair.clone(),
                            dispatcher: params.dispatcher.clone(),
                            chain_id: params.chain_id.clone(),
                            adjust_block_gas_limit_fn: None,
                            confirmation_count: params.confirmation_count,
                        },
                        ethereum.clone(),
                        ethereum_monitor.clone(),
                        pool.clone(),
                        gas_pricer.clone(),
                    );
                    info!(target: "relayer",
                          "Relayer service: relayer machine {:?} created", keypair.address());
                    machine
                })
                .collect(),
        );

        {
            let mut pool = pool.lock();
            pool.set_relayers(
                params
                    .relayer_keypairs
                    .iter()
                    .map(KeyPair::address)
                    .collect(),
            );
            pool.set_dispatcher(params.dispatcher.address().clone());
        }

        trace!(target: "relayer",
            "Relayer service: {:?} relayer machine created!",
            relayer_machines.lock().len()
        );

        Service {
            running: false,
            ethereum,
            ethereum_monitor,
            pool,
            gas_pricer,

            relayer_machines,
            retired_relayer_machines: Mutex::new(HashSet::new()),

            ticker: Interval::new_interval(params.interval),
        }
    }

    fn relayer_status(&self, relayer_address: &Address) -> Option<RelayerState> {
        self.relayer_machines
            .lock()
            .iter()
            .find(|machine| machine.address().eq(relayer_address))
            .map(RelayerMachine::state)
    }

    fn poll_ticker(&mut self) -> Poll<Option<()>, Error> {
        match self.ticker.poll() {
            Ok(Async::Ready(Some(_))) => {
                trace!(target: "relayer",
                        "Relayer service: timeouts, try to create token transfer request transaction");

                let ready_count = self.pool.lock().count_by_tag(PoolRequestTag::Ready);
                if ready_count > 0 {
                    // notify all idle relayer machine
                    let selected =
                        self.relayer_machines
                            .lock()
                            .iter_mut()
                            .fold(0, |selected, relayer| {
                                if relayer.state() == RelayerState::Ready {
                                    info!(target: "relayer",
                                        "Relayer service: relayer {:?} is selected to relay token transfer request",
                                        relayer.address()
                                    );
                                    relayer.send_event(RelayerEvent::Timeout);
                                    selected + 1
                                } else {
                                    selected
                                }
                            });

                    if 0 == selected {
                        info!(target: "relayer",
                            "Relayer service: no availible relayer!");
                    }
                } else {
                    trace!(target: "relayer",
                        "Relayer service: no ready in request pool, skip!");
                }
            }
            Err(err) => return Err(Error::from(err)),
            _ => {}
        }
        Ok(Async::NotReady)
    }

    fn poll_relayer(&mut self) -> Poll<Option<()>, Error> {
        for relayer in self.relayer_machines.lock().iter_mut() {
            match relayer.poll() {
                Ok(Async::Ready(Some(RelayerState::Ready))) => {
                    let requests = self.pool.lock().remove_by_tag(PoolRequestTag::Executed);
                    info!(target: "relayer",
                        "Relayer service: {} executed request(s) removed from pool by relayer {}",
                        requests.len(), relayer.address());
                }
                Ok(Async::Ready(Some(RelayerState::Preparing))) => {}
                Ok(Async::Ready(Some(RelayerState::GasEstimating))) => {}
                Ok(Async::Ready(Some(RelayerState::TxBroadcasting))) => {}
                Ok(Async::Ready(Some(RelayerState::TxExecuting))) => {}
                Ok(Async::Ready(None)) => {}
                Ok(Async::NotReady) => {}
                Err(err) => {
                    let relayer_address = relayer.address();
                    warn!(target: "relayer",
                        "Relayer service: relayer {:?} occur error: {:?}, reset relayer {:?}",
                        relayer_address, err, relayer_address,
                    );
                    relayer.reset();
                }
            }
        }

        Ok(Async::NotReady)
    }

    fn remove_retired_relayers(&mut self) {
        let mut retired = self.retired_relayer_machines.lock();
        let mut machines = self.relayer_machines.lock();

        let to_remove: HashSet<_> = retired
            .iter()
            .cloned()
            .filter(|relayer_address| {
                self.relayer_status(relayer_address) == Some(RelayerState::Ready)
            })
            .collect();

        info!(target: "relayer", "Relayer service: Remove retired relayer machine(s): {:?}", to_remove);
        machines.retain(|machine| !to_remove.contains(&machine.address()));

        retired.retain(|relayer_address| !to_remove.contains(relayer_address));
    }
}

impl<E, M, P, G, C> traits::MachineService for Service<E, M, P, G, C>
where
    E: EthereumService<Error = EthereumServiceError>,
    M: EthereumMonitor<MonitorError = EthereumMonitorError>,
    P: PoolService<SignedRequest = SignedRequest, Hash = H256>,
    G: PriceService<PricerError = PriceServiceError>,
    C: RequestConverter,
{
    type MachineError = Error;
    type MachineParams = RelayerParams<C>;
    type MachineStatus = Status;
    type SignedRequest = SignedRequest;
    type RelayerMode = RelayerMode;
    type RelayerInfo = RelayerInfo;

    #[inline]
    fn is_working(&self) -> bool {
        self.running
    }

    fn start(&mut self) -> bool {
        if self.running {
            return true;
        }
        self.running = true;
        info!(target: "relayer", "Relayer service: Start relayer service");
        self.is_working()
    }

    fn stop(&mut self) -> bool {
        if !self.running {
            return self.is_working();
        }
        self.running = false;
        info!(target: "relayer", "Relayer service: Stop relayer service");
        self.is_working()
    }

    fn set_dispatcher_address(&mut self, address: Address) {
        self.relayer_machines
            .lock()
            .iter_mut()
            .for_each(|machine| machine.set_dispatcher_address(address.clone()));
    }

    fn set_chain_id(&mut self, chain_id: Option<u64>) {
        self.relayer_machines
            .lock()
            .iter_mut()
            .for_each(|machine| machine.set_chain_id(chain_id));
    }

    fn set_confirmation_count(&mut self, confirmation_count: u32) {
        self.relayer_machines
            .lock()
            .iter_mut()
            .for_each(|machine| machine.set_confirmation_count(confirmation_count));
    }

    fn set_interval(&mut self, interval: Duration) -> Result<Duration, Self::MachineError> {
        if interval == Duration::from_secs(0) {
            warn!(target: "relayer", "Relayer service: Invalid interval value: {:?}", interval);
            return Err(Error::from(ErrorKind::InvalidIntervalValue(interval)));
        }

        info!(target: "relayer", "Relayer service: Set interval to {:?}", interval);
        self.ticker = Interval::new_interval(interval);
        Ok(interval)
    }

    fn force_relay(
        &mut self,
        signed_request: Self::SignedRequest,
    ) -> Box<Future<Item = Arc<Self::SignedRequest>, Error = Self::MachineError> + Send> {
        match self
            .relayer_machines
            .lock()
            .iter_mut()
            .find(|machine| machine.state() == RelayerState::Ready)
        {
            Some(relayer) => {
                info!(target: "relayer",
                    "Relayer service: Force relay single token transfer request {:?} with relayer {:?}",
                    signed_request.hash(),
                    relayer.address()
                );
                relayer.send_event(RelayerEvent::SingleRequest(signed_request.clone()));
                Box::new(futures::future::ok(Arc::new(signed_request)))
            }
            None => {
                info!(target: "relayer",
                    "Relayer service: No available relayer, import token transfer request {:?} into pool",
                    signed_request.hash()
                );

                Box::new(
                    self.pool
                        .lock()
                        .import(signed_request)
                        .map_err(|_| Error::from(ErrorKind::FailedToImportTokenTransferRequest)),
                )
            }
        }
    }

    fn contains_relayer(&self, relayer_address: &Address) -> bool {
        self.relayer_machines
            .lock()
            .iter()
            .find(|machine| machine.address().eq(relayer_address))
            .is_some()
    }

    fn add_relayer(
        &mut self,
        mode: Self::RelayerMode,
        params: Self::MachineParams,
    ) -> Result<Option<RelayerInfo>, Self::MachineError> {
        let address = params.keypair.address();
        if self.contains_relayer(&address) {
            return Ok(self.relayer_info(&address));
        }

        let (relayer_info, relayers) = {
            let machine = RelayerMachine::new(
                mode,
                params,
                self.ethereum.clone(),
                self.ethereum_monitor.clone(),
                self.pool.clone(),
                self.gas_pricer.clone(),
            );
            let info = machine.info();
            let mut machines = self.relayer_machines.lock();
            machines.push(machine);

            (info, machines.iter().map(RelayerMachine::address).collect())
        };

        self.pool.lock().set_relayers(relayers);
        Ok(Some(relayer_info))
    }

    fn remove_relayer(&mut self, address: &Address) -> Result<(), Self::MachineError> {
        match self.relayer_status(address) {
            None => return Ok(()),
            Some(RelayerState::Ready) => {
                info!(target: "relayer", "Relayer service: Remove relayer machine: {:?}", address);
                let relayers = {
                    let mut machines = self.relayer_machines.lock();
                    machines.retain(|machine| !machine.address().eq(address));
                    machines.iter().map(RelayerMachine::address).collect()
                };
                self.pool.lock().set_relayers(relayers);
            }
            Some(state) => {
                info!(target: "relayer",
                    "Relayer service: Mark relayer machine: {:?} as retired, current state: {}",
                    address, state.to_string());
                self.retired_relayer_machines.lock().insert(address.clone());

                let mut relayers = self.relayers();
                relayers.remove(&address);
                self.pool.lock().set_relayers(relayers);
            }
        }

        if self.relayer_count() == 0 {
            warn!(target: "relayer", "Relayer service: All relayer is removed, we can not relay any token transfer request");
        }

        Ok(())
    }

    fn relayer_mode(&self, relayer_address: &Address) -> Option<RelayerMode> {
        self.relayer_machines
            .lock()
            .iter()
            .find(|machine| machine.address().eq(relayer_address))
            .map(RelayerMachine::mode)
    }

    fn set_relayer_mode(
        &mut self,
        relayer_address: &Address,
        mode: RelayerMode,
    ) -> Option<RelayerMode> {
        self.relayer_machines
            .lock()
            .iter_mut()
            .find(|machine| machine.address().eq(relayer_address))
            .map(|machine| {
                machine.set_mode(mode);
                machine.mode()
            })
    }

    fn status(&self) -> Self::MachineStatus {
        let mut relayer_infos: Vec<RelayerInfo> = self
            .relayer_machines
            .lock()
            .iter()
            .map(RelayerMachine::info)
            .collect();

        relayer_infos.sort_unstable();
        Status {
            is_working: self.is_working(),
            relayer_infos,
            relayer_count: traits::MachineService::relayer_count(self),
        }
    }

    #[inline]
    fn relayer_info(&self, relayer_address: &Address) -> Option<Self::RelayerInfo> {
        self.relayer_machines
            .lock()
            .iter()
            .find(|machine| machine.address().eq(relayer_address))
            .map(RelayerMachine::info)
    }

    #[inline]
    fn relayers(&self) -> HashSet<Address> {
        self.relayer_machines
            .lock()
            .iter()
            .map(RelayerMachine::address)
            .collect()
    }

    #[inline]
    fn relayer_count(&self) -> usize {
        self.relayer_machines.lock().len()
    }

    #[inline]
    fn dispatcher_contracts(&self) -> Vec<Address> {
        use std::collections::HashSet;
        let dispatchers: HashSet<_> = self
            .relayer_machines
            .lock()
            .iter()
            .map(RelayerMachine::dispatcher_address)
            .collect();

        dispatchers.into_iter().collect()
    }
}

impl<E, M, P, G, C> Stream for Service<E, M, P, G, C>
where
    E: EthereumService<Error = EthereumServiceError>,
    M: EthereumMonitor<MonitorError = EthereumMonitorError>,
    P: PoolService<SignedRequest = SignedRequest, Hash = H256>,
    G: PriceService<PricerError = PriceServiceError>,
    C: RequestConverter,
{
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            if let Err(err) = self.poll_relayer() {
                return Err(err);
            }

            self.remove_retired_relayers();

            if !self.running {
                return Ok(Async::NotReady);
            }

            if let Err(err) = self.poll_ticker() {
                return Err(err);
            }

            return Ok(Async::NotReady);
        }
    }
}
