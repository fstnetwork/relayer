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
use crate::types::SignedRequest;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RelayerEvent {
    Null,
    Timeout,
    Thredshold,
    SingleRequest(SignedRequest),
}

impl RelayerEvent {
    #[allow(unused)]
    pub fn is_null(&self) -> bool {
        *self == RelayerEvent::Null
    }

    #[allow(unused)]
    pub fn is_timeout(&self) -> bool {
        *self == RelayerEvent::Timeout
    }

    #[allow(unused)]
    pub fn is_thredshold(&self) -> bool {
        *self == RelayerEvent::Thredshold
    }

    #[allow(unused)]
    pub fn is_single_request(&self) -> bool {
        match *self {
            RelayerEvent::SingleRequest(_) => true,
            _ => false,
        }
    }
}
