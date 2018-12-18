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
use ethcore_transaction;

use ethereum_types::{Address, H256, U256};
use futures::{sync::mpsc, Future, Stream};

use types::{
    AccountState, BlockId, Currency, EthRpcTransaction, EthRpcTransactionReceipt,
    EthTransactionConfirmation, GasEstimation,
};

pub type AccountStateFuture<Error> = Box<Future<Item = AccountState, Error = Error> + Send>;
pub type BytesFuture<Error> = Box<Future<Item = Vec<u8>, Error = Error> + Send>;
pub type BoolFuture<Error> = Box<Future<Item = bool, Error = Error> + Send>;
pub type U256Future<Error> = Box<Future<Item = U256, Error = Error> + Send>;
pub type H256Future<Error> = Box<Future<Item = H256, Error = Error> + Send>;
pub type EthRpcTransactionFuture<Error> =
    Box<Future<Item = Option<EthRpcTransaction>, Error = Error> + Send>;
pub type EthRpcTransactionReceiptFuture<Error> =
    Box<Future<Item = Option<EthRpcTransactionReceipt>, Error = Error> + Send>;
pub type EthRpcTransactionConfirmationFuture<Error> =
    Box<Future<Item = EthTransactionConfirmation, Error = Error> + Send>;

pub trait BlockInfoProvider<Error>: Send + Sync {
    fn get_block_number(&self) -> U256Future<Error>;
}

pub trait TransactionFetcher<Error>: Send + Sync {
    fn get_transaction_by_hash(&self, tx_hash: H256) -> EthRpcTransactionFuture<Error>;
    fn get_transaction_receipt(&self, tx_hash: H256) -> EthRpcTransactionReceiptFuture<Error>;
    fn get_transaction_confirmation(
        &self,
        tx_hash: H256,
    ) -> EthRpcTransactionConfirmationFuture<Error>;
}

pub trait TransactionBroadcaster<Error>: Send + Sync {
    fn send_transaction(&self, tx: ethcore_transaction::SignedTransaction) -> H256Future<Error>;
}

pub trait GasEstimator<Error>: Send + Sync {
    fn block_gas_limit(&self, block_id: BlockId) -> U256Future<Error>;
    fn estimate_gas(&self, gas_estimate: GasEstimation) -> U256Future<Error>;
}

pub trait AccountStateProvider<Error>: Send + Sync {
    fn balance_of(&self, account: Address, currency: Currency) -> U256Future<Error>;
    fn nonce_of(&self, account: Address, currency: Currency) -> U256Future<Error>;
    fn state_of(&self, account: Address, currency: Currency) -> AccountStateFuture<Error>;
    fn code_of(&self, account: Address) -> BytesFuture<Error>;
}

pub trait TokenStateProvider<Error>: Send + Sync {
    fn token_delegate_enable(&self, token_contract: Address) -> BoolFuture<Error>;
}

pub trait EthereumService:
    Send
    + Sync
    + AccountStateProvider<<Self as EthereumService>::Error>
    + TokenStateProvider<<Self as EthereumService>::Error>
    + BlockInfoProvider<<Self as EthereumService>::Error>
    + GasEstimator<<Self as EthereumService>::Error>
    + TransactionBroadcaster<<Self as EthereumService>::Error>
    + TransactionFetcher<<Self as EthereumService>::Error>
{
    type Error: ::std::error::Error + Send + 'static;

    /// Adds new Ethereum endpoint
    fn add_endpoint(&mut self, endpoint: String) -> bool;

    /// Removes a Ethereum endpoint
    fn remove_endpoint(&mut self, endpoint: &String) -> bool;

    fn contains_endpoint(&mut self, endpoint: &String) -> bool;

    /// Returns current endpoints used by Ethereum Service
    fn endpoints(&self) -> Vec<String>;

    /// Ruturns current endpoints count used by Ethereum Service
    fn endpoint_count(&self) -> usize;
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum EthereumMonitorTask {
    TransactionExecuted { confirmation_count: u32, hash: H256 },
    BlockNumberReached(U256),
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum EthereumMonitorResponse {
    BlockHash(H256),
    BlockNumber(U256),
    Transaction(H256),
    Request(H256),
}

pub trait EthereumMonitor: Send + Sync + Stream {
    type MonitorError: ::std::error::Error + Send + 'static;
    type WatcherId: Send + Sync + Copy + Eq + PartialEq;

    fn register(
        &mut self,
    ) -> (
        Self::WatcherId,
        mpsc::UnboundedReceiver<EthereumMonitorResponse>,
    );
    fn register_and_subscribe(
        &mut self,
        task: EthereumMonitorTask,
    ) -> (
        Self::WatcherId,
        mpsc::UnboundedReceiver<EthereumMonitorResponse>,
    );
    fn unregister(&mut self, watcher_id: Self::WatcherId);

    fn subscribe(
        &mut self,
        watcher_id: Self::WatcherId,
        task: EthereumMonitorTask,
    ) -> Result<(), Self::MonitorError>;
    fn unsubscribe(
        &mut self,
        watcher_id: Self::WatcherId,
        task: &EthereumMonitorTask,
    ) -> Result<(), Self::MonitorError>;
}
