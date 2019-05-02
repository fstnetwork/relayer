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

use super::{Bytes, H160, U256};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CallRequest {
    pub to: Option<H160>,

    pub from: Option<H160>,

    pub gas: Option<U256>,

    #[serde(rename = "gasPrice")]
    pub gas_price: Option<U256>,

    pub value: Option<U256>,

    pub data: Option<Bytes>,
}

impl Default for CallRequest {
    fn default() -> CallRequest {
        CallRequest {
            to: None,
            from: None,
            gas: None,
            gas_price: None,
            value: None,
            data: None,
        }
    }
}
