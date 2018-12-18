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
use jsonrpc_core::Result;
use machine::MachineStatus;

build_rpc_trait! {
    pub trait RelayerApi {

        #[rpc(name = "relayer_addRelayer")]
        fn add_relayer(&self, String) -> Result<bool>;

        #[rpc(name="relayer_removeRelayer")]
        fn remove_relayer(&self, String) -> Result<bool>;

        #[rpc(name = "relayer_relayers")]
        fn relayers(&self) -> Result<Vec<types::EthRpcH160>>;

        #[rpc(name = "relayer_relayerCount")]
        fn relayer_count(&self) -> Result<usize>;

        #[rpc(name = "relayer_dispatcherContracts")]
        fn dispatcher_contracts(&self) -> Result<Vec<types::EthRpcH160>>;

        #[rpc(name = "relayer_start")]
        fn start(&self) -> Result<bool>;

        #[rpc(name = "relayer_stop")]
        fn stop(&self) -> Result<bool>;

        #[rpc(name = "relayer_setInterval")]
        fn set_interval(&self, u64) -> Result<u64>;

        #[rpc(name = "relayer_isWorking")]
        fn is_working(&self) -> Result<bool>;

        #[rpc(name = "relayer_status")]
        fn status(&self) -> Result<MachineStatus>;
    }
}
