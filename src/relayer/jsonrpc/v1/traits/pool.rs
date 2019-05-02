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

build_rpc_trait! {
    pub trait PoolApi {

        #[rpc(name="pool_version")]
        fn version(&self) -> Result<String>;

        #[rpc(name="pool_allRequests")]
        fn all_requests(&self) -> Result<Vec<crate::types::RelayerRpcRequest>>;

        #[rpc(name="pool_requestCount")]
        fn request_count(&self) -> Result<usize>;

        #[rpc(name="pool_readyRequests")]
        fn ready_requests(&self) -> Result<Vec<crate::types::RelayerRpcRequest>>;

        #[rpc(name="pool_futureRequests")]
        fn future_requests(&self) -> Result<Vec<crate::types::RelayerRpcRequest>>;
    }
}
