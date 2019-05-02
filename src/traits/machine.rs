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

use ethereum_types::Address;
use futures::{Future, Stream};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

pub trait MachineService: Sync + Send + Stream {
    type MachineError: ::std::error::Error + Send + 'static + ToString;
    type MachineParams;
    type MachineStatus;
    type SignedRequest;
    type RelayerMode;
    type RelayerInfo;

    /// Starts relayer machine service
    fn start(&mut self) -> bool;

    /// Stops relayer machine service
    fn stop(&mut self) -> bool;

    /// Sets relay interval and returns new relay interval.
    /// # Arguments
    ///
    /// * `interval` - new relay interval
    ///
    fn set_interval(&mut self, interval: Duration) -> Result<Duration, Self::MachineError>;

    /// Returns if relayer machine service is working or not
    fn is_working(&self) -> bool;

    fn force_relay(
        &mut self,
        signed_request: Self::SignedRequest,
    ) -> Box<Future<Item = Arc<Self::SignedRequest>, Error = Self::Error> + Send>;

    /// Adds new relayer account and returns relayer info
    fn add_relayer(
        &mut self,
        mode: Self::RelayerMode,
        params: Self::MachineParams,
    ) -> Result<Option<Self::RelayerInfo>, Self::MachineError>;

    /// Removes a relayer from relayer service machine
    fn remove_relayer(&mut self, address: &Address) -> Result<(), Self::MachineError>;

    fn contains_relayer(&self, address: &Address) -> bool;

    fn relayer_mode(&self, relayer_address: &Address) -> Option<Self::RelayerMode>;

    fn set_relayer_mode(
        &mut self,
        relayer_address: &Address,
        mode: Self::RelayerMode,
    ) -> Option<Self::RelayerMode>;

    /// Set confirmation count for all relayers
    fn set_confirmation_count(&mut self, confirmation_count: u32);

    /// Set chain id for all relayers
    fn set_chain_id(&mut self, chain_id: Option<u64>);

    /// Set dispatcher address for all relayers
    fn set_dispatcher_address(&mut self, address: Address);

    fn status(&self) -> Self::MachineStatus;

    fn relayer_count(&self) -> usize;

    fn relayers(&self) -> HashSet<Address>;

    fn relayer_info(&self, relayer_address: &Address) -> Option<Self::RelayerInfo>;

    fn dispatcher_contracts(&self) -> Vec<Address>;
}
