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
use jsonrpc_core::Result;

use super::traits::SystemInfoApi;

pub struct SystemInfo {
    name: String,
    version: String,
}

impl SystemInfo {
    pub fn new() -> SystemInfo {
        SystemInfo {
            name: "relayer".to_owned(),
            version: "0.0.1".to_owned(),
        }
    }

    pub fn name(mut self, name: String) -> SystemInfo {
        self.name = name;
        self
    }

    pub fn version(mut self, version: String) -> SystemInfo {
        self.version = version;
        self
    }
}

impl SystemInfoApi for SystemInfo {
    fn name(&self) -> Result<String> {
        Ok(self.name.clone())
    }

    fn version(&self) -> Result<String> {
        Ok(self.version.clone())
    }
}
