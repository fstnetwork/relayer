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
use futures::{Async, Future, Poll, Stream};
use parking_lot::Mutex;
use std::sync::Arc;

use service::{ExitReason, Service};

pub type ExitCode = i32;

pub struct ExitStatus {
    pub code: ExitCode,
    pub reason: ExitReason,
}

pub struct Server {
    service: Arc<Mutex<Service>>,
}

impl Server {
    pub fn new(service: Arc<Mutex<Service>>) -> Server {
        Server { service }
    }
}

impl Future for Server {
    type Item = ExitStatus;
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            match self.service.lock().poll() {
                Ok(Async::Ready(Some(reason))) => {
                    return Ok(Async::Ready(ExitStatus { reason, code: 0 }));
                }
                Ok(Async::Ready(None)) | Ok(Async::NotReady) => return Ok(Async::NotReady),
                Err(err) => return Err(err),
            }
        }
    }
}
