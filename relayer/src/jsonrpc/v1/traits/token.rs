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
use jsonrpc_core::{BoxFuture, Result};

build_rpc_trait! {
    pub trait TokenApi {
        #[rpc(name="token_isDelegateEnable")]
        fn is_delegate_enable(&self, Address) -> BoxFuture<bool>;

        #[rpc(name="token_balanceOf")]
        fn balance_of(&self, Address, Address) -> BoxFuture<U256>;

        #[rpc(name="token_nonceOf")]
        fn nonce_of(&self, Address, Address) -> BoxFuture<U256>;

        #[rpc(name="token_sendTokenTransferRequest")]
        fn send_token_transfer_request(&self, types::RelayerRpcRequest) -> BoxFuture<H256>;

        #[rpc(name="token_sendTokenTransferRequests")]
        fn send_token_transfer_requests(&self, Vec<types::RelayerRpcRequest>) -> BoxFuture<Vec<H256>>;

        #[rpc(name="token_supportedTokens")]
        fn supported_tokens(&self) -> Result<Vec<types::RelayerRpcToken>>;
    }
}
