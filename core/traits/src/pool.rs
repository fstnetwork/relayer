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
use futures::{Future, Stream};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PoolRequestTag {
    Invalid,
    Ready,
    Processing,
    Executed,
}

pub struct PoolPendingSettings {
    pub filter: Option<Box<Fn(&Address) -> bool>>,
    pub gas_limit: U256,
    pub relayer: Option<Address>,
}

pub trait PoolService: Sync + Send + Stream {
    type SignedRequest;
    type Address;
    type Hash;
    type Filter: Clone;
    type PoolParams;
    type PoolStatus;
    type PoolError: ::std::error::Error + Send + ToString + 'static;

    fn import(
        &mut self,
        request: Self::SignedRequest,
    ) -> Box<Future<Item = Arc<Self::SignedRequest>, Error = Self::PoolError> + Send>;

    fn all_requests(&self) -> Vec<Arc<Self::SignedRequest>>;

    fn ready_requests(
        &mut self,
        new_tag: Option<PoolRequestTag>,
        pending_settings: PoolPendingSettings,
    ) -> Vec<Arc<Self::SignedRequest>>;

    fn contains_hash(&self, hash: &Self::Hash) -> bool;

    fn mark_by_hash(&mut self, hash: &Self::Hash, tag: PoolRequestTag);

    fn mark_by_hashes(&mut self, hash: &[Self::Hash], tag: PoolRequestTag);

    fn remove_by_hash(&mut self, hash: &Self::Hash) -> Option<Arc<Self::SignedRequest>>;

    fn remove_by_hashes(&mut self, hash: &[Self::Hash]) -> Vec<Arc<Self::SignedRequest>>;

    fn remove_by_token(&mut self, tokens: &Self::Address) -> Option<Arc<Self::SignedRequest>>;

    fn remove_by_tokens(&mut self, tokens: &[Self::Address]) -> Vec<Arc<Self::SignedRequest>>;

    fn remove_by_tag(&mut self, tag: PoolRequestTag) -> Vec<Arc<Self::SignedRequest>>;

    fn remove_by_sender(&mut self, senders: &Self::Address) -> Option<Arc<Self::SignedRequest>>;

    fn remove_by_senders(&mut self, senders: &[Self::Address]) -> Vec<Arc<Self::SignedRequest>>;

    fn filter(&self) -> Self::Filter;

    fn set_filter(&mut self, filter: Self::Filter);

    fn clear(&mut self);

    fn len(&self) -> usize;

    fn count_by_tag(&self, tag: PoolRequestTag) -> usize;

    fn tags(&self) -> HashMap<Self::Hash, PoolRequestTag>;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn token_status(
        &self,
        token_address: &Self::Address,
    ) -> Result<Self::PoolStatus, Self::PoolError>;

    fn status(&self) -> HashMap<Self::Address, Self::PoolStatus>;

    fn set_interval(&mut self, interval: Duration);

    fn set_params(&mut self, params: Self::PoolParams);

    fn set_relayers(&mut self, relayers: HashSet<Address>);

    fn set_dispatcher(&mut self, dispatcher: Address);
}
