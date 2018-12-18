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
extern crate ethcore_transaction;
extern crate ethereum_types;
extern crate futures;

extern crate types;

mod ethereum;
mod machine;
mod network;
mod pool;
mod pricer;

pub use self::ethereum::{
    AccountStateFuture, BoolFuture, BytesFuture, EthRpcTransactionConfirmationFuture,
    EthRpcTransactionFuture, EthRpcTransactionReceiptFuture, H256Future, U256Future,
};
pub use self::ethereum::{
    AccountStateProvider, BlockInfoProvider, EthereumMonitor, EthereumMonitorResponse,
    EthereumMonitorTask, EthereumService, GasEstimator, TokenStateProvider, TransactionBroadcaster,
    TransactionFetcher,
};
pub use self::machine::MachineService;
pub use self::network::NetworkService;
pub use self::pool::{PoolPendingSettings, PoolRequestTag, PoolService};
pub use self::pricer::PriceService;

pub trait ExitHandle: Send + Sync + Clone {
    fn shutdown(&mut self);
}
