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
use ethereum_types::U256;
use futures::{Async, Future, IntoFuture, Poll};
use parking_lot::Mutex;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::timer::Delay;

use traits::{EthereumMonitorResponse, EthereumMonitorTask, EthereumService};

pub struct TaskPoller<E>
where
    E: EthereumService,
{
    ethereum: Arc<Mutex<E>>,
    task: EthereumMonitorTask,
    interval: Duration,
    future: Box<Future<Item = Option<EthereumMonitorResponse>, Error = ()> + Send>,
}

unsafe impl<E> Sync for TaskPoller<E> where E: EthereumService {}
unsafe impl<E> Send for TaskPoller<E> where E: EthereumService {}

impl<E> Eq for TaskPoller<E> where E: EthereumService {}

impl<E> TaskPoller<E>
where
    E: EthereumService,
{
    pub fn new(
        ethereum: Arc<Mutex<E>>,
        task: EthereumMonitorTask,
        interval: Duration,
    ) -> TaskPoller<E> {
        let future = Self::new_future(ethereum.clone(), task.clone());
        TaskPoller {
            ethereum,
            task,
            interval,
            future,
        }
    }

    fn new_future(
        ethereum: Arc<Mutex<E>>,
        task: EthereumMonitorTask,
    ) -> Box<Future<Item = Option<EthereumMonitorResponse>, Error = ()> + Send> {
        trace!(target: "ethereum_monitor", "create new polling future for task {:?}", task);
        match task {
            EthereumMonitorTask::TransactionExecuted {
                hash,
                confirmation_count,
            } => Box::new(
                ethereum
                    .lock()
                    .get_transaction_confirmation(hash)
                    .map({
                        move |confirmation| match confirmation.receipt {
                            Some(receipt) => {
                                let receipt_block_number = receipt.block_number?;
                                let tx_hash = receipt.transaction_hash?;

                                match confirmation.block_number - receipt_block_number
                                    >= U256::from(confirmation_count)
                                {
                                    true => {
                                        Some(EthereumMonitorResponse::Transaction(tx_hash.into()))
                                    }
                                    false => None,
                                }
                            }
                            None => None,
                        }
                    })
                    .map_err(|_| ()),
            ) as Box<_>,
            EthereumMonitorTask::BlockNumberReached(block_number) => Box::new(
                ethereum
                    .lock()
                    .get_block_number()
                    .map({
                        let expected_block_number = block_number;
                        move |latest_block_number| match latest_block_number
                            >= expected_block_number
                        {
                            true => Some(EthereumMonitorResponse::BlockNumber(latest_block_number)),
                            false => None,
                        }
                    })
                    .map_err(|_| ()),
            ) as Box<_>,
        }
    }

    pub fn task(&self) -> &EthereumMonitorTask {
        &self.task
    }
}

impl<E> Future for TaskPoller<E>
where
    E: EthereumService,
{
    type Item = EthereumMonitorResponse;
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.future.poll() {
            Ok(Async::Ready(response)) => match response {
                Some(response) => Ok(Async::Ready(response)),
                None => {
                    let future = Self::new_future(self.ethereum.clone(), self.task.clone());
                    let delay = Instant::now() + self.interval;
                    self.future =
                        Box::new(Delay::new(delay).then(move |_| future).into_future()) as Box<_>;
                    Ok(Async::NotReady)
                }
            },
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(err) => Err(err),
        }
    }
}

impl<E> Hash for TaskPoller<E>
where
    E: EthereumService,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.task.hash(state)
    }
}

impl<E> PartialEq for TaskPoller<E>
where
    E: EthereumService,
{
    fn eq(&self, other: &TaskPoller<E>) -> bool {
        self.task == other.task
    }
}
