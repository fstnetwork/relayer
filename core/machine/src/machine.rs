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
use ethereum_types::{Address, H256, U256};
use ethkey::KeyPair;
use futures::{sync::mpsc, Async, Future, Poll, Stream};
use parking_lot::Mutex;
use std::cmp;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use ethcore_transaction::SignedTransaction;

use collation::{ClosedCollation, OpenCollation, RequestConverter, RequestDispatcher};
use ethereum::{monitor::Error as EthereumMonitorError, service::Error as EthereumServiceError};
use pricer::Error as PriceServiceError;
use traits::{
    EthereumMonitor, EthereumMonitorResponse, EthereumMonitorTask, EthereumService, PoolRequestTag,
    PoolService, PriceService,
};
use types::{AccountState, BlockId, Currency, GasEstimation, SignedRequest};

use super::{Error, ErrorKind, RelayerEvent};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RelayerMode {
    BroadcastTransaction,
    NotBroadcastTransaction,
}

pub struct RelayerParams<C>
where
    C: RequestConverter,
{
    pub keypair: KeyPair,
    pub dispatcher: RequestDispatcher<C>,
    pub chain_id: Option<u64>,
    pub adjust_block_gas_limit_fn: Option<fn(U256) -> U256>,
    pub confirmation_count: u32,
}

#[derive(PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub enum RelayerState {
    #[serde(rename = "ready")]
    Ready,

    #[serde(rename = "preparing")]
    Preparing,

    #[serde(rename = "gasEstimating")]
    GasEstimating,

    #[serde(rename = "transactionBroadcasting")]
    TxBroadcasting,

    #[serde(rename = "transactionExecuting")]
    TxExecuting,
}

impl ToString for RelayerState {
    fn to_string(&self) -> String {
        match *self {
            RelayerState::Ready => "Ready".to_owned(),
            RelayerState::Preparing => "Preparing".to_owned(),
            RelayerState::GasEstimating => "Gas estimating".to_owned(),
            RelayerState::TxBroadcasting => "Transaction broadcasting".to_owned(),
            RelayerState::TxExecuting => "Transaction executing".to_owned(),
        }
    }
}

struct Preparation {
    account_state: AccountState,
    block_gas_limit: U256,
    gas_price: U256,
}

impl Default for Preparation {
    fn default() -> Preparation {
        Preparation {
            account_state: AccountState::empty(),
            block_gas_limit: U256::from(8_000_000 * 2 / 3),
            gas_price: U256::zero(),
        }
    }
}

type InfoFetcher = Box<Future<Item = Preparation, Error = Error> + Send>;
type TxBroadcaster = Box<Future<Item = H256, Error = Error> + Send>;
type GasEstimator = Box<Future<Item = ClosedCollation, Error = Error> + Send>;

enum StateWorker {
    Ready,
    Prepare {
        event: RelayerEvent,
        fetcher: InfoFetcher,
    },
    GasEstimation(GasEstimator),
    BroadcastTransaction(TxBroadcaster),
    WaitingTransactionExecuted,
}

#[derive(Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct RelayerInfo {
    // relayer's address
    address: Address,

    #[serde(rename = "dispatcherAddress")]
    // dispatcher contract address
    dispatcher_address: Address,

    // current state of a relayer machine
    state: RelayerState,
}

impl cmp::Ord for RelayerInfo {
    fn cmp(&self, other: &RelayerInfo) -> cmp::Ordering {
        self.address.cmp(&other.address)
    }
}

impl cmp::PartialOrd for RelayerInfo {
    fn partial_cmp(&self, other: &RelayerInfo) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct RelayerMachine<E, M, P, G, C>
where
    E: EthereumService<Error = EthereumServiceError>,
    M: EthereumMonitor<MonitorError = EthereumMonitorError>,
    P: PoolService<SignedRequest = SignedRequest, Hash = H256>,
    G: PriceService<PricerError = PriceServiceError>,
    C: RequestConverter,
{
    ethereum: Arc<Mutex<E>>,
    ethereum_monitor: Arc<Mutex<M>>,
    pool: Arc<Mutex<P>>,
    gas_pricer: Arc<Mutex<G>>,

    mode: RelayerMode,
    state: RelayerState,
    state_worker: StateWorker,

    current_collation: Option<ClosedCollation>,

    keypair: KeyPair,
    relayer_address: Address,

    dispatcher: RequestDispatcher<C>,
    chain_id: Option<u64>,

    confirmation_count: u32,
    adjust_block_gas_limit_fn: fn(U256) -> U256,
    adjust_gas_amount_fn: fn(U256, U256) -> U256,

    event_sender: mpsc::UnboundedSender<RelayerEvent>,
    event_receiver: mpsc::UnboundedReceiver<RelayerEvent>,

    monitor_receiver: mpsc::UnboundedReceiver<EthereumMonitorResponse>,
    monitor_wather_id: M::WatcherId,
}

impl<E, M, P, G, C> RelayerMachine<E, M, P, G, C>
where
    E: EthereumService<Error = EthereumServiceError>,
    M: EthereumMonitor<MonitorError = EthereumMonitorError>,
    P: PoolService<SignedRequest = SignedRequest, Hash = H256>,
    G: PriceService<PricerError = PriceServiceError>,
    C: RequestConverter,
{
    pub fn adjust_block_gas_limit(original: U256) -> U256 {
        // original * U256::from(2) / U256::from(3)
        original - U256::from(100000)
    }

    pub fn adjust_gas_amount(gas_amount: U256, max_gas_limit: U256) -> U256 {
        // gas amount for a transaction must lower than max block gas limit

        let adjusted = U256::min(
            gas_amount + (gas_amount / U256::from(10)) + U256::from(50000),
            max_gas_limit,
        );
        info!(target: "relayer",
            "Orignal estimated gas amount: {}, adjusted estimated gas amount: {}",
            gas_amount, adjusted
        );
        adjusted
    }

    fn state_transfer(&mut self, new_state: RelayerState, new_state_worker: StateWorker) {
        info!(target: "relayer",
            "relayer {} state transfer from {:?} to {:?}",
            self.relayer_address,
            self.state.to_string(),
            new_state.to_string()
        );

        self.state = new_state;
        self.state_worker = new_state_worker;
    }

    pub fn new(
        mode: RelayerMode,
        params: RelayerParams<C>,
        ethereum: Arc<Mutex<E>>,
        ethereum_monitor: Arc<Mutex<M>>,
        pool: Arc<Mutex<P>>,
        gas_pricer: Arc<Mutex<G>>,
    ) -> RelayerMachine<E, M, P, G, C> {
        let keypair = params.keypair;
        let address = keypair.address();

        let (monitor_wather_id, monitor_receiver) = ethereum_monitor.lock().register();
        let state_worker = StateWorker::Ready;
        let (event_sender, event_receiver) = mpsc::unbounded();
        let adjust_block_gas_limit_fn = match params.adjust_block_gas_limit_fn {
            Some(cal) => cal,
            None => Self::adjust_block_gas_limit,
        };

        RelayerMachine {
            ethereum,
            ethereum_monitor,
            pool,
            gas_pricer,

            mode,
            state: RelayerState::Ready,
            state_worker,

            current_collation: None,

            keypair,
            relayer_address: address,

            confirmation_count: params.confirmation_count,
            adjust_block_gas_limit_fn,
            adjust_gas_amount_fn: Self::adjust_gas_amount,

            dispatcher: params.dispatcher,
            chain_id: params.chain_id,

            event_sender,
            event_receiver,

            monitor_receiver,
            monitor_wather_id,
        }
    }

    fn new_info_fetcher(
        ethereum: Arc<Mutex<E>>,
        gas_pricer: Arc<Mutex<G>>,
        address: Address,
    ) -> InfoFetcher {
        enum Info {
            AccountState(AccountState),
            GasLimit(U256),
            GasPrice(U256),
        }
        type InfoFuture = Box<Future<Item = Info, Error = Error> + Send>;

        let (state, gas_limit) = {
            let ethereum = ethereum.lock();
            let state: InfoFuture = Box::new(
                ethereum
                    .state_of(address, Currency::Ether)
                    .and_then(|account_state| Ok(Info::AccountState(account_state)))
                    .from_err(),
            );
            let gas_limit: InfoFuture = Box::new(
                ethereum
                    .block_gas_limit(BlockId::Latest)
                    .and_then(|gas_limit| Ok(Info::GasLimit(gas_limit)))
                    .from_err(),
            );
            (state, gas_limit)
        };
        let gas_price: InfoFuture = Box::new(
            gas_pricer
                .lock()
                .gas_price()
                .and_then(|gas_price| Ok(Info::GasPrice(gas_price)))
                .from_err(),
        );

        Box::new(
            futures::future::join_all(vec![state, gas_limit, gas_price]).map(|results| {
                results
                    .into_iter()
                    .fold(Preparation::default(), |mut preparation, value| {
                        match value {
                            Info::AccountState(state) => {
                                preparation.account_state = state;
                            }
                            Info::GasLimit(gas_limit) => {
                                preparation.block_gas_limit = gas_limit;
                            }
                            Info::GasPrice(gas_price) => {
                                preparation.gas_price = gas_price;
                            }
                        }

                        preparation
                    })
            }),
        )
    }

    pub fn reset(&mut self) {
        info!(target: "relayer",
            "relayer {} reset machine state from {:?} to {:?}",
            self.relayer_address,
            self.state.to_string(),
            RelayerState::Ready.to_string()
        );

        if let Some(ref mut current_collation) =
            ::std::mem::replace(&mut self.current_collation, None)
        {
            info!(target: "relayer",
                  "relayer {} reset {} request(s) from {:?} to {:?}",
                  self.relayer_address, current_collation.request_count(),
                  PoolRequestTag::Processing, PoolRequestTag::Ready);
            self.pool
                .lock()
                .mark_by_hashes(&current_collation.request_hashes(), PoolRequestTag::Ready);
        }

        self.state = RelayerState::Ready;
        self.state_worker = StateWorker::Ready;
    }

    #[inline]
    pub fn mode(&self) -> RelayerMode {
        self.mode
    }

    #[inline]
    pub fn set_mode(&mut self, mode: RelayerMode) {
        self.mode = mode;
    }

    #[inline]
    pub fn address(&self) -> Address {
        self.relayer_address.clone()
    }

    #[inline]
    pub fn dispatcher_address(&self) -> Address {
        self.dispatcher.address().clone()
    }

    #[inline]
    pub fn state(&self) -> RelayerState {
        self.state
    }

    #[allow(unused)]
    #[inline]
    pub fn current_collation(&self) -> Option<ClosedCollation> {
        self.current_collation.clone()
    }

    #[allow(unused)]
    #[inline]
    pub fn current_transaction(&self) -> Option<SignedTransaction> {
        self.current_collation
            .as_ref()
            .map(|collation| collation.transaction().clone())
    }

    #[inline]
    pub fn send_event(&mut self, event: RelayerEvent) {
        self.event_sender
            .unbounded_send(event)
            .expect("receiver always existed; qed");
    }

    #[inline]
    pub fn event_sender(&self) -> mpsc::UnboundedSender<RelayerEvent> {
        self.event_sender.clone()
    }

    #[inline]
    pub fn change_keypair(&mut self, keypair: KeyPair) {
        self.keypair = keypair;
    }

    #[inline]
    pub fn change_dispatcher(&mut self, dispatcher: RequestDispatcher<C>) {
        self.dispatcher = dispatcher;
    }

    #[inline]
    pub fn set_dispatcher_address(&mut self, dispatcher_address: Address) {
        self.dispatcher.set_address(dispatcher_address);
    }

    #[inline]
    pub fn set_confirmation_count(&mut self, confirmation_count: u32) {
        self.confirmation_count = confirmation_count;
    }

    #[inline]
    pub fn set_chain_id(&mut self, chain_id: Option<u64>) {
        self.chain_id = chain_id;
    }

    fn select_event(events: &Vec<RelayerEvent>, default_event: RelayerEvent) -> RelayerEvent {
        if events.is_empty() || events.iter().all(|event| *event == RelayerEvent::Null) {
            return RelayerEvent::Null;
        }

        match events.iter().find(|event| event.is_single_request()) {
            Some(event) => event.clone(),
            None => default_event,
        }
    }

    fn poll_ready(&mut self) -> Poll<Option<RelayerState>, Error> {
        assert!(self.current_collation.is_none());

        let event = Self::select_event(
            &vec![match self.event_receiver.poll().unwrap() {
                Async::Ready(Some(event)) => event,
                _ => RelayerEvent::Null,
            }],
            RelayerEvent::Timeout,
        );

        if event.is_null() {
            return Ok(Async::NotReady);
        }

        let fetcher = Self::new_info_fetcher(
            self.ethereum.clone(),
            self.gas_pricer.clone(),
            self.address(),
        );
        self.state_transfer(
            RelayerState::Preparing,
            StateWorker::Prepare { event, fetcher },
        );

        Ok(Async::Ready(Some(self.state)))
    }

    fn poll_preparing(&mut self) -> Poll<Option<RelayerState>, Error> {
        assert!(self.current_collation.is_none());

        let (event, info) = if let StateWorker::Prepare {
            ref event,
            ref mut fetcher,
        } = self.state_worker
        {
            match fetcher.poll() {
                Ok(Async::Ready(info)) => (event.clone(), info),
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                Err(err) => return Err(err),
            }
        } else {
            return Err(Error::from(ErrorKind::InvalidStateTransfer(
                self.state.to_string(),
                RelayerState::Preparing.to_string(),
            )));
        };

        assert_ne!(event, RelayerEvent::Null);
        let Preparation {
            account_state,
            block_gas_limit,
            gas_price,
        } = info;

        info!(target: "relayer",
            "relayer {:?} balance: {}, nonce: {}",
            account_state.address(),
            account_state.balance(),
            account_state.nonce()
        );
        info!(target: "relayer", "latest block gas limit: {:?}", block_gas_limit);

        let mut open_collation = match event {
            RelayerEvent::Null => return Ok(Async::NotReady),
            RelayerEvent::SingleRequest(request) => OpenCollation::with_single_request(request),
            RelayerEvent::Timeout | RelayerEvent::Thredshold => {
                let gas_limit = (self.adjust_block_gas_limit_fn)(block_gas_limit);

                let requests: Vec<_> = {
                    let mut pool = self.pool.lock();
                    let requests: Vec<_> = pool
                        .ready_requests(
                            Some(PoolRequestTag::Processing),
                            traits::PoolPendingSettings {
                                filter: None,
                                gas_limit,
                                relayer: Some(self.relayer_address),
                            },
                        )
                        .iter()
                        .map(|req| req.as_ref().clone())
                        .collect();

                    requests
                };

                if requests.is_empty() {
                    info!(target: "relayer",
                        "relayer {}: no ready token transfer request in pool",
                        self.relayer_address
                    );

                    // transfer machine state back to ready
                    self.state_transfer(RelayerState::Ready, StateWorker::Ready);

                    return Ok(Async::Ready(Some(self.state)));
                }

                info!(target: "relayer", "relayer {} try to relay {} request(s)",
                    self.relayer_address, requests.len());
                OpenCollation::with_requests(requests)
            }
        };

        let value = U256::zero();
        open_collation.update_unestimated(
            &self.dispatcher,
            &account_state.nonce(),
            &gas_price,
            &value,
        );
        let unestimated_tx = match open_collation.unestimated() {
            Some(tx) => tx.clone(),
            None => return Err(Error::from(ErrorKind::EmptyTokenTransferRequestTransaction)),
        };

        {
            let fake_closed_collation = open_collation
                .fake_close(self.keypair.secret().clone(), self.chain_id)
                .expect("keypair is valid; qed");

            self.current_collation = Some(fake_closed_collation);
        }

        let new_state_worker = StateWorker::GasEstimation(Box::new(
            self.ethereum
                .lock()
                .estimate_gas(GasEstimation::Transaction(
                    unestimated_tx.fake_sign(self.keypair.address()),
                ))
                .from_err::<Error>()
                .and_then({
                    let secret = self.keypair.secret().clone();
                    let chain_id = self.chain_id;
                    let adjust_gas_amount_fn = self.adjust_gas_amount_fn;

                    move |gas_amount| {
                        Ok(open_collation.close_with_gas(
                            secret,
                            chain_id,
                            &adjust_gas_amount_fn(gas_amount, block_gas_limit),
                        )?)
                    }
                })
                .from_err::<Error>(),
        ));

        self.state_transfer(RelayerState::GasEstimating, new_state_worker);
        Ok(Async::Ready(Some(self.state)))
    }

    fn poll_estimating(&mut self) -> Poll<Option<RelayerState>, Error> {
        assert!(match self.current_collation {
            Some(ref c) => c.is_fake(),
            None => false,
        });

        let closed_collation =
            if let StateWorker::GasEstimation(ref mut estimate_future) = self.state_worker {
                match estimate_future.poll() {
                    Ok(Async::Ready(closed_collation)) => closed_collation,
                    Ok(Async::NotReady) => return Ok(Async::NotReady),
                    Err(err) => return Err(err),
                }
            } else {
                return Err(Error::from(ErrorKind::InvalidStateTransfer(
                    self.state.to_string(),
                    RelayerState::GasEstimating.to_string(),
                )));
            };

        let signed_tx = closed_collation.transaction().clone();
        {
            let (tx, _, _) = signed_tx.clone().deconstruct();
            let request_count = closed_collation.request_count();
            info!(target: "relayer",
                "relayer {:?} current tx: {:?}, nonce: {}, gas price: {}, gas: {}, requests: {}",
                self.address(),
                tx.hash(),
                tx.nonce,
                tx.gas_price,
                tx.gas,
                request_count,
            );
        }

        match self.mode {
            RelayerMode::BroadcastTransaction => {
                // update current collation
                self.current_collation = Some(closed_collation);

                info!(target: "relayer",
                    "relayer {} is broadcasting transaction {:?}",
                    self.relayer_address,
                    signed_tx.hash()
                );

                // transfer machine state from gas estimating to transaction broadcasting
                let new_state_worker = StateWorker::BroadcastTransaction(Box::new(
                    self.ethereum
                        .lock()
                        .send_transaction(signed_tx)
                        .and_then(Ok)
                        .from_err(),
                ));
                self.state_transfer(RelayerState::TxBroadcasting, new_state_worker);
            }
            RelayerMode::NotBroadcastTransaction => {
                // make sure current collation is none
                self.current_collation = None;

                info!(target: "relayer",
                    "relayer mode = {:?}, relayer {} will not broadcasting transaction {:?}",
                    self.mode,
                    self.relayer_address,
                    signed_tx.hash()
                );

                // transfer machine state back to ready
                self.state_transfer(RelayerState::Ready, StateWorker::Ready);
            }
        }

        Ok(Async::Ready(Some(self.state.clone())))
    }

    fn poll_broadcasting(&mut self) -> Poll<Option<RelayerState>, Error> {
        assert!(self.current_collation.is_some());

        let tx_hash = if let StateWorker::BroadcastTransaction(ref mut broadcast_future) =
            self.state_worker
        {
            match broadcast_future.poll() {
                Ok(Async::Ready(tx_hash)) => tx_hash,
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                Err(err) => return Err(err),
            }
        } else {
            return Err(Error::from(ErrorKind::InvalidStateTransfer(
                self.state.to_string(),
                RelayerState::TxBroadcasting.to_string(),
            )));
        };

        self.state_transfer(
            RelayerState::TxExecuting,
            StateWorker::WaitingTransactionExecuted,
        );

        return match self.ethereum_monitor.lock().subscribe(
            self.monitor_wather_id,
            EthereumMonitorTask::TransactionExecuted {
                hash: tx_hash,
                confirmation_count: self.confirmation_count,
            },
        ) {
            Ok(_) => Ok(Async::Ready(Some(self.state))),
            Err(err) => Err(Error::from(err)),
        };
    }

    fn poll_executing(&mut self) -> Poll<Option<RelayerState>, Error> {
        assert!(self.current_collation.is_some());

        let tx_hash = match self.monitor_receiver.poll().unwrap() {
            Async::Ready(Some(EthereumMonitorResponse::Transaction(tx_hash))) => tx_hash,
            _ => return Ok(Async::NotReady),
        };

        assert_eq!(
            tx_hash,
            match self.current_collation {
                Some(ref closed_collation) => closed_collation.transaction().hash(),
                None => H256::zero(),
            }
        );

        {
            let current_collation = ::std::mem::replace(&mut self.current_collation, None);
            let current_collation = current_collation.expect("current_collation is some; qed");
            let hashes = current_collation.request_hashes();
            info!(target: "relayer",
                  "relayer {} mark {} requests as {:?}",
                  self.relayer_address, hashes.len(), PoolRequestTag::Executed );
            self.pool
                .lock()
                .mark_by_hashes(&hashes, PoolRequestTag::Executed);
        }

        self.state_transfer(RelayerState::Ready, StateWorker::Ready);
        Ok(Async::Ready(Some(self.state)))
    }

    #[inline]
    pub fn info(&self) -> RelayerInfo {
        RelayerInfo {
            address: self.address(),
            dispatcher_address: self.dispatcher_address(),
            state: self.state(),
        }
    }
}

impl<E, M, P, G, C> Stream for RelayerMachine<E, M, P, G, C>
where
    E: EthereumService<Error = EthereumServiceError>,
    M: EthereumMonitor<MonitorError = EthereumMonitorError>,
    P: PoolService<SignedRequest = SignedRequest, Hash = H256>,
    G: PriceService<PricerError = PriceServiceError>,
    C: RequestConverter,
{
    type Item = RelayerState;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.state {
            RelayerState::Ready => self.poll_ready(),
            RelayerState::Preparing => self.poll_preparing(),
            RelayerState::GasEstimating => self.poll_estimating(),
            RelayerState::TxBroadcasting => self.poll_broadcasting(),
            RelayerState::TxExecuting => self.poll_executing(),
        }
    }
}

impl<E, M, P, G, C> Hash for RelayerMachine<E, M, P, G, C>
where
    E: EthereumService<Error = EthereumServiceError>,
    M: EthereumMonitor<MonitorError = EthereumMonitorError>,
    P: PoolService<SignedRequest = SignedRequest, Hash = H256>,
    G: PriceService<PricerError = PriceServiceError>,
    C: RequestConverter,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.relayer_address.hash(state);
    }
}

impl<E, M, P, G, C> PartialEq for RelayerMachine<E, M, P, G, C>
where
    E: EthereumService<Error = EthereumServiceError>,
    M: EthereumMonitor<MonitorError = EthereumMonitorError>,
    P: PoolService<SignedRequest = SignedRequest, Hash = H256>,
    G: PriceService<PricerError = PriceServiceError>,
    C: RequestConverter,
{
    fn eq(&self, other: &RelayerMachine<E, M, P, G, C>) -> bool {
        self.relayer_address.eq(&other.relayer_address)
    }
}

impl<E, M, P, G, C> Eq for RelayerMachine<E, M, P, G, C>
where
    E: EthereumService<Error = EthereumServiceError>,
    M: EthereumMonitor<MonitorError = EthereumMonitorError>,
    P: PoolService<SignedRequest = SignedRequest, Hash = H256>,
    G: PriceService<PricerError = PriceServiceError>,
    C: RequestConverter,
{
}
