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
use futures::{sync::mpsc, Async, Future, Poll, Stream};
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use traits::{EthereumMonitor, EthereumMonitorResponse, EthereumMonitorTask, EthereumService};

use super::{Error, ErrorKind, TaskPoller, WatcherId};

struct Watcher {
    tasks: HashSet<EthereumMonitorTask>,
    sender: mpsc::UnboundedSender<EthereumMonitorResponse>,
}

pub struct Params {
    pub ticker_interval: Duration,
}

pub struct Service<E>
where
    E: EthereumService,
{
    ethereum: Arc<Mutex<E>>,
    ticker_interval: Duration,
    watchers: HashMap<WatcherId, Watcher>,
    watcher_counter: WatcherId,
    task_pollers: HashMap<EthereumMonitorTask, TaskPoller<E>>,
}

impl<E> Service<E>
where
    E: EthereumService,
{
    pub fn new(ethereum: Arc<Mutex<E>>, params: Params) -> Service<E> {
        Service {
            ethereum,
            ticker_interval: params.ticker_interval,

            watchers: HashMap::default(),
            watcher_counter: 0,
            task_pollers: HashMap::new(),
        }
    }

    fn add_task(&mut self, task: EthereumMonitorTask) {
        if self.task_pollers.contains_key(&task) {
            return;
        }

        self.task_pollers.insert(
            task,
            TaskPoller::new(self.ethereum.clone(), task, self.ticker_interval),
        );
    }

    #[inline]
    fn remove_task(&mut self, task: &EthereumMonitorTask) {
        self.task_pollers.remove(task);
    }

    #[inline]
    pub fn ticker_interval(&self) -> Duration {
        self.ticker_interval
    }
}

impl<E> EthereumMonitor for Service<E>
where
    E: EthereumService,
{
    type WatcherId = WatcherId;
    type MonitorError = Error;

    fn register(&mut self) -> (WatcherId, mpsc::UnboundedReceiver<EthereumMonitorResponse>) {
        let (sender, receiver) = mpsc::unbounded();

        let watcher_id = self.watcher_counter;
        self.watcher_counter += 1;

        self.watchers.insert(
            watcher_id,
            Watcher {
                tasks: HashSet::new(),
                sender,
            },
        );

        (watcher_id, receiver)
    }

    fn register_and_subscribe(
        &mut self,
        task: EthereumMonitorTask,
    ) -> (WatcherId, mpsc::UnboundedReceiver<EthereumMonitorResponse>) {
        let (watcher_id, receiver) = self.register();

        match self.watchers.get_mut(&watcher_id) {
            Some(watcher) => {
                watcher.tasks.insert(task.clone());
            }
            None => panic!(format!("watcher {:?} does not exist", watcher_id)),
        }

        self.add_task(task);
        (watcher_id, receiver)
    }

    fn unregister(&mut self, _watcher_id: WatcherId) {
        // TODO
        unimplemented!();
    }

    fn subscribe(&mut self, watcher_id: WatcherId, task: EthereumMonitorTask) -> Result<(), Error> {
        match self.watchers.get_mut(&watcher_id) {
            None => return Err(Error::from(ErrorKind::InvalidWatcherId)),
            Some(watcher) => {
                watcher.tasks.insert(task);
            }
        }

        self.add_task(task);
        Ok(())
    }

    fn unsubscribe(
        &mut self,
        watcher_id: WatcherId,
        task: &EthereumMonitorTask,
    ) -> Result<(), Error> {
        let removed = match self.watchers.get_mut(&watcher_id) {
            Some(watcher) => watcher.tasks.remove(task),
            None => return Err(Error::from(ErrorKind::InvalidWatcherId)),
        };

        if removed {
            let should_remove_task = !self
                .watchers
                .iter()
                .any(|(_, watcher)| watcher.tasks.contains(task));

            if should_remove_task {
                self.remove_task(task);
            }
        }

        Ok(())
    }
}

impl<E> Stream for Service<E>
where
    E: EthereumService,
{
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let (response_pairs, err_indexes) = self.task_pollers.iter_mut().fold(
            (HashMap::new(), Vec::new()),
            |(mut response_pairs, mut err_indexes), (task, task_poller)| {
                match task_poller.poll() {
                    Ok(Async::NotReady) => {}
                    Ok(Async::Ready(response)) => {
                        response_pairs.insert(task.clone(), response);
                    }
                    Err(_err) => {
                        // TODO error handling
                        err_indexes.push(task.clone());
                    }
                };
                (response_pairs, err_indexes)
            },
        );

        self.task_pollers
            .retain(|task, _| !response_pairs.contains_key(&task) && !err_indexes.contains(&task));

        let mut to_remove = Vec::new();
        for (task, response) in response_pairs.iter() {
            for (id, watcher) in self.watchers.iter() {
                if watcher.tasks.contains(task) {
                    if watcher.sender.unbounded_send(response.clone()).is_err() {
                        to_remove.push(*id);
                    }
                }
            }
        }

        to_remove.into_iter().for_each(|ref id| {
            self.watchers.remove(id);
        });

        Ok(Async::NotReady)
    }
}
