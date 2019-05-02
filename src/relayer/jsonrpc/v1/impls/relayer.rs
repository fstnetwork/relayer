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
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;

use jsonrpc_core::{Error, ErrorCode, Result};

use crate::machine::MachineStatus;

use super::traits::RelayerApi;

pub struct Relayer<M>
where
    M: 'static + crate::traits::MachineService<MachineStatus = crate::machine::MachineStatus>,
{
    machine: Arc<Mutex<M>>,
}

impl<M> Relayer<M>
where
    M: 'static + crate::traits::MachineService<MachineStatus = crate::machine::MachineStatus>,
{
    pub fn new(machine: Arc<Mutex<M>>) -> Relayer<M> {
        Relayer { machine }
    }
}

impl<M> RelayerApi for Relayer<M>
where
    M: 'static + crate::traits::MachineService<MachineStatus = crate::machine::MachineStatus>,
{
    fn add_relayer(&self, _private_key: String) -> Result<bool> {
        Ok(true)
    }

    fn remove_relayer(&self, _address: String) -> Result<bool> {
        Ok(true)
    }

    fn relayers(&self) -> Result<Vec<crate::types::EthRpcH160>> {
        let relayers = self.machine.lock().relayers();
        Ok(relayers
            .into_iter()
            .map(crate::types::EthRpcH160::from)
            .collect())
    }

    fn relayer_count(&self) -> Result<usize> {
        Ok(self.machine.lock().relayer_count())
    }

    fn dispatcher_contracts(&self) -> Result<Vec<crate::types::EthRpcH160>> {
        let dispatcher_contracts = self.machine.lock().dispatcher_contracts();
        Ok(dispatcher_contracts
            .into_iter()
            .map(crate::types::EthRpcH160::from)
            .collect())
    }

    fn start(&self) -> Result<bool> {
        Ok(self.machine.lock().start())
    }

    fn stop(&self) -> Result<bool> {
        Ok(self.machine.lock().stop())
    }

    fn set_interval(&self, interval_secs: u64) -> Result<u64> {
        match self
            .machine
            .lock()
            .set_interval(Duration::from_secs(interval_secs))
        {
            Ok(interval) => Ok(interval.as_secs()),
            Err(err) => Err(Error {
                code: ErrorCode::InvalidParams,
                message: err.to_string(),
                data: None,
            }),
        }
    }

    fn is_working(&self) -> Result<bool> {
        Ok(self.machine.lock().is_working())
    }

    fn status(&self) -> Result<MachineStatus> {
        Ok(self.machine.lock().status())
    }
}
