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
use futures::{Async, Poll, Stream};
use std::time::Duration;
use tokio_timer::Interval;

use crate::traits;

use super::error::Error;

pub struct Params {}

pub struct Service {
    ticker: Interval,
}

impl Service {
    pub fn new(_params: Params) -> Service {
        // TODO
        Service {
            ticker: Interval::new_interval(Duration::from_millis(100)),
        }
    }
}

impl Drop for Service {
    fn drop(&mut self) {}
}

impl traits::NetworkService for Service {
    type NetworkError = Error;

    fn peer_count(&self) -> usize {
        0
    }

    fn is_listening(&self) -> bool {
        false
    }
}

impl Stream for Service {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            match self.ticker.poll().unwrap() {
                Async::Ready(_) => {
                    // trace!(target: "network", "network timeouts");
                    // add routine works here
                }
                _ => {
                    return Ok(Async::NotReady);
                }
            }
        }
    }
}
