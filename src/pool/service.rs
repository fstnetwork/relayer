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
use futures::{Async, Future, Poll, Stream};
use parking_lot::{Mutex, RwLock};
use std::collections::{HashMap, HashSet};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;
use tokio_timer::Interval;

use crate::traits;
use crate::types::{DelegateMode, SignedRequest};

use super::{
    AddressFilter, Error, ErrorKind, InnerPool, PoolParams, PoolRequest, PoolRequestTag, Readiness,
    RequestSelector, Status, Verifier,
};

pub struct Service<E, N, F, R, S, V>
where
    E: traits::EthereumService,
    N: traits::NetworkService,
    F: AddressFilter + Send,
    R: PoolRequest + Send + Sync + 'static,
    S: RequestSelector<R> + Send + Sync + 'static,
    V: Verifier<Request = R, Error = Error> + Send,
{
    /// Ethereum Service instance
    ethereum: Arc<Mutex<E>>,

    /// Network Service instance
    network: Arc<Mutex<N>>,

    /// inner pool instance
    inner: Arc<RwLock<InnerPool<R, S>>>,

    /// Filter is used to filtering token
    token_filter: Mutex<F>,

    /// Verifier is used to filtering token transfer request when importing token transfer request
    verifier: Arc<Mutex<V>>,

    /// current insertion id
    insertion_id: Arc<AtomicUsize>,

    /// relayer addresses
    relayers: HashSet<Address>,

    /// dispatcher contract address
    dispatcher: Address,

    /// ticker for routine jobs
    ticker: Interval,
}

impl<E, N, F, R, S, V> Service<E, N, F, R, S, V>
where
    E: traits::EthereumService,
    N: traits::NetworkService,
    F: AddressFilter + Send,
    R: PoolRequest + Send + Sync + 'static,
    S: RequestSelector<R> + Send + Sync + 'static,
    V: Verifier<Request = R, Error = Error> + Send,
{
    /// Returns a Pool Service
    ///
    /// # Arguments
    ///
    /// * `params` - Parameters of inner pool
    /// * `interval` - Polling interval for routine jobs
    /// * `ethereum` - Ethereum Service instance
    /// * `network` - Network Service instance
    /// * `token_filter` - Token filter for filtering `token` field of a token transfer request
    /// * `verfier` - Token transfer request verifier
    /// * `selector` - Selector for choosing token transfer request when importing new token
    /// transfer request
    ///
    pub fn new(
        params: PoolParams,
        interval: Duration,
        ethereum: Arc<Mutex<E>>,
        network: Arc<Mutex<N>>,
        token_filter: F,
        verifier: Arc<Mutex<V>>,
        selector: Arc<S>,
    ) -> Service<E, N, F, R, S, V> {
        Service {
            ethereum,
            network,
            inner: Arc::new(RwLock::new(InnerPool::new(params, selector))),
            token_filter: Mutex::new(token_filter),
            verifier,
            insertion_id: Arc::new(AtomicUsize::new(0)),

            relayers: Default::default(),
            dispatcher: Default::default(),

            ticker: Interval::new_interval(interval),
        }
    }
}

impl<E, N, F, R, S, V> traits::PoolService for Service<E, N, F, R, S, V>
where
    E: traits::EthereumService,
    N: traits::NetworkService,
    F: AddressFilter + Send,
    R: PoolRequest + Send + Sync + 'static,
    S: RequestSelector<R> + Send + Sync + 'static,
    V: Verifier<Request = R, Error = Error> + Send,
{
    type SignedRequest = SignedRequest;
    type Address = Address;
    type Hash = H256;
    type Filter = F;
    type PoolParams = PoolParams;
    type PoolStatus = Status;
    type PoolError = Error;

    fn import(
        &mut self,
        request: SignedRequest,
    ) -> Box<Future<Item = Arc<Self::SignedRequest>, Error = Self::PoolError> + Send> {
        if self.inner.read().contains_hash(&request.hash()) {
            info!(
                "reject an already imported token transfer request {:?}",
                request.hash()
            );
            return Box::new(futures::future::ok(Arc::new(request)));
        }

        if !self
            .token_filter
            .lock()
            .is_allowed(&request.unverified().token())
        {
            return Box::new(futures::future::err(Error::from(
                ErrorKind::NotSupportedToken,
            )));
        }

        let relayer_address = {
            match request.unverified().delegate_mode() {
                DelegateMode::PublicMsgSender | DelegateMode::PublicTxOrigin => Address::zero(),
                DelegateMode::PrivateMsgSender | DelegateMode::PrivateTxOrigin => {
                    let relayer_address = request.unverified().relayer_address().clone();

                    // check if we can execute this request
                    if !self.relayers.contains(&relayer_address)
                        && self.dispatcher != relayer_address
                    {
                        // TODO broadcast to p2p network
                        return Box::new(futures::future::ok(Arc::new(request)));
                    }
                    relayer_address
                }
            }
        };

        let insertion_id = self.insertion_id.fetch_add(1, Ordering::Relaxed);

        Box::new(
            self.verifier
                .lock()
                .verify_request(request, insertion_id, relayer_address, self.inner.clone())
                .from_err(),
        )
    }

    fn ready_requests(
        &mut self,
        new_tag: Option<PoolRequestTag>,
        pending_settings: traits::PoolPendingSettings,
    ) -> Vec<Arc<SignedRequest>> {
        let traits::PoolPendingSettings {
            ref relayer,
            gas_limit,
            ..
        } = pending_settings;

        let mut pool = self.inner.write();
        // let _senders_with_token = pool.senders_with_token();

        let ready = |_request: &R| {
            // TODO check if token transfer request is executable
            Readiness::Ready
        };

        let mut total_gas_amount = U256::from(0);
        let requests: Vec<_> = {
            let tags = pool.tags();
            pool.pending(ready)
                .filter(|req| {
                    let hash = req.hash();
                    match tags.get(hash) {
                        Some(PoolRequestTag::Ready) => {
                            // this request is ready
                        }
                        _ => {
                            // this request is not tagged as ready
                            return false;
                        }
                    }

                    if req.delegate_mode().is_private() {
                        if let Some(relayer) = relayer {
                            if relayer != req.relayer() && !self.dispatcher.eq(req.relayer()) {
                                return false;
                            }
                        }
                    }

                    if gas_limit == 0.into() {
                        return true;
                    }

                    let gas_amount = req.gas_amount();
                    if total_gas_amount + gas_amount <= gas_limit {
                        total_gas_amount += *gas_amount;

                        return true;
                    }
                    false
                })
                .map(|req| req.clone_signed())
                .collect()
        };

        if let Some(new_tag) = new_tag {
            let hashes: Vec<_> = requests.iter().map(|req| *req.hash()).collect();
            pool.mark_by_hashes(&hashes, new_tag);
        }

        requests
    }

    fn contains_hash(&self, hash: &H256) -> bool {
        self.inner.read().contains_hash(hash)
    }

    fn all_requests(&self) -> Vec<Arc<SignedRequest>> {
        let ready = |_request: &R| Readiness::Ready;
        self.inner
            .read()
            .unordered_pending(ready)
            .map(|req| req.clone_signed())
            .collect()
    }

    #[inline]
    fn mark_by_hash(&mut self, hash: &H256, tag: PoolRequestTag) {
        self.inner.write().mark_by_hash(hash, tag);
    }

    #[inline]
    fn mark_by_hashes(&mut self, hashes: &[H256], tag: PoolRequestTag) {
        self.inner.write().mark_by_hashes(hashes, tag);
    }

    #[inline]
    fn remove_by_hash(&mut self, hash: &H256) -> Option<Arc<SignedRequest>> {
        self.inner.write().remove(hash)
    }

    #[inline]
    fn remove_by_hashes(&mut self, hashes: &[H256]) -> Vec<Arc<SignedRequest>> {
        let mut pool = self.inner.write();
        hashes.iter().fold(Vec::new(), |mut v, hash| {
            if let Some(req) = pool.remove(hash) {
                v.push(req);
            }
            v
        })
    }

    #[inline]
    fn remove_by_tag(&mut self, tag: PoolRequestTag) -> Vec<Arc<Self::SignedRequest>> {
        let mut pool = self.inner.write();
        let tags = pool.tags().clone();
        pool.remove_by_filter(|req: &R| tags.get(req.hash()) == Some(&tag))
    }

    #[inline]
    fn remove_by_token(&mut self, token: &Address) -> Option<Arc<SignedRequest>> {
        self.inner
            .write()
            .remove_by_filter(|req: &R| token == req.token())
            .pop()
    }

    #[inline]
    fn remove_by_tokens(&mut self, tokens: &[Address]) -> Vec<Arc<SignedRequest>> {
        self.inner
            .write()
            .remove_by_filter(|req: &R| tokens.contains(req.token()))
    }

    #[inline]
    fn remove_by_sender(&mut self, sender: &Address) -> Option<Arc<SignedRequest>> {
        self.inner
            .write()
            .remove_by_filter(|req: &R| sender == req.sender())
            .pop()
    }

    #[inline]
    fn remove_by_senders(&mut self, senders: &[Address]) -> Vec<Arc<SignedRequest>> {
        self.inner
            .write()
            .remove_by_filter(|req: &R| senders.contains(req.sender()))
    }

    #[inline]
    fn filter(&self) -> Self::Filter {
        self.token_filter.lock().clone()
    }

    #[inline]
    fn set_filter(&mut self, filter: Self::Filter) {
        self.inner
            .write()
            .remove_by_filter(|req: &R| filter.is_denied(req.token()));
        self.token_filter = Mutex::new(filter);
    }

    #[inline]
    fn len(&self) -> usize {
        self.inner.write().len()
    }

    #[inline]
    fn count_by_tag(&self, tag: PoolRequestTag) -> usize {
        self.inner.write().count_by_tag(tag)
    }

    #[inline]
    fn tags(&self) -> HashMap<H256, PoolRequestTag> {
        self.inner.write().tags().clone()
    }

    #[inline]
    fn clear(&mut self) {
        self.inner.write().clear();
    }

    fn token_status(&self, token_address: &Address) -> Result<Status, Error> {
        match self.token_filter.lock().is_allowed(&token_address) {
            true => Ok(self.inner.write().token_status(token_address)),
            false => Err(Error::from(ErrorKind::NotSupportedToken)),
        }
    }

    #[inline]
    fn status(&self) -> HashMap<Address, Status> {
        self.inner.write().status()
    }

    #[inline]
    fn set_interval(&mut self, interval: Duration) {
        self.ticker = Interval::new_interval(interval);
    }

    #[inline]
    fn set_params(&mut self, params: Self::PoolParams) {
        self.inner.write().set_params(params);
    }

    #[inline]
    fn set_relayers(&mut self, relayers: HashSet<Address>) {
        self.relayers = relayers;
    }

    #[inline]
    fn set_dispatcher(&mut self, dispatcher: Address) {
        self.dispatcher = dispatcher;
    }
}

impl<E, N, F, R, S, V> Stream for Service<E, N, F, R, S, V>
where
    E: traits::EthereumService,
    N: traits::NetworkService,
    F: AddressFilter + Send,
    R: PoolRequest + Send + Sync,
    S: RequestSelector<R> + Send,
    V: Verifier<Request = R, Error = Error> + Send,
{
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            match self.ticker.poll() {
                Ok(Async::Ready(_)) => {
                    // trace!(target: "pool", "pool timeouts");
                    // TODO add pool routine jobs here
                    // self.inner.lock().remove_stalled
                }
                Ok(Async::NotReady) => {
                    return Ok(Async::NotReady);
                }
                Err(err) => {
                    return Err(Error::from(err));
                }
            }
        }
    }
}
