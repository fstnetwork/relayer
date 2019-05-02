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

use ethcore::transaction::SignedTransaction;

mod ethereum_rpc;
mod relayer_rpc;
mod request;
pub mod solidity;
mod state;

pub use self::ethereum_rpc::{
    EthRpcBytes, EthRpcCallRequest, EthRpcH160, EthRpcH256, EthRpcH512, EthRpcTransaction,
    EthRpcTransactionReceipt, EthRpcU128, EthRpcU256, EthRpcU64,
};
pub use self::relayer_rpc::{RelayerRpcRequest, RelayerRpcToken};
pub use self::request::{
    signature, DelegateMode, Request, RequestError, SignedRequest, UnverifiedRequest,
};
pub use self::state::AccountState;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockId {
    Number(U256),
    Hash(H256),
    Latest,
    Pending,
    Earliest,
}

impl ToString for BlockId {
    fn to_string(&self) -> String {
        match *self {
            BlockId::Pending => "pending".to_owned(),
            BlockId::Latest => "latest".to_owned(),
            BlockId::Earliest => "earliest".to_owned(),
            BlockId::Number(n) => n.to_string(),
            BlockId::Hash(hash) => hash.to_string(),
        }
    }
}

impl Default for BlockId {
    fn default() -> BlockId {
        BlockId::Number(U256::from(0))
    }
}

pub struct EthTransactionConfirmation {
    pub block_number: U256,
    pub receipt: Option<EthRpcTransactionReceipt>,
}

#[derive(Debug, Clone)]
pub enum Currency {
    Ether,
    Token(Address),
}

#[derive(Debug, Clone)]
pub enum GasEstimation {
    Transaction(SignedTransaction),
    TokenTransferRequest {
        relayer_address: Address,
        signed_request: SignedRequest,
    },
}
