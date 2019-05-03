use ethereum_types::{Address, U256};
use futures::{Async, Future, Poll, Stream};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tokio::timer::Interval;

use super::contract::Erc1376Contract;
use super::error::Error;

pub struct Service {
    ticker: Interval,
    state: State,
    // block_future:
    current_block_number: U256,
    cache: Cache,
    token_contracts: HashSet<Address>,
}

#[derive(Copy, Clone, Debug)]
pub enum State {
    Stopped,
    Syncing,
    Listening,
}

struct Cache {
    contracts: HashMap<Address, Erc1376Contract>,
}

impl Service {
    pub fn new(interval: Duration) -> Service {
        Service {
            ticker: Interval::new_interval(interval),
            current_block_number: U256::zero(),
            state: State::Stopped,
            token_contracts: Default::default(),
            cache: Cache {
                contracts: Default::default(),
            },
        }
    }

    pub fn start(&mut self) {}

    pub fn stop(&mut self) {}

    pub fn state(&self) -> State {
        self.state
    }

    pub fn set_interval(&mut self, interval: Duration) -> Result<(), Error> {
        if interval == Duration::from_nanos(0) {
            return Err(Error::InvalidInterval(interval));
        }

        self.ticker = Interval::new_interval(interval);
        Ok(())
    }
}

impl Stream for Service {
    type Item = U256;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            match self.ticker.poll() {
                Ok(Async::Ready(_)) => {}
                Err(err) => return Err(Error::from(err)),
                _ => {}
            }

            return Ok(Async::NotReady);
        }
    }
}
