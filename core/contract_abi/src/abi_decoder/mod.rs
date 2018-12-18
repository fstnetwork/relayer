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
mod dispatcher;
mod token;

pub use self::dispatcher::FstTokenTransferRequestDispatcherAbiDecoder;

pub use self::token::{ERC1376AbiDecoder, ERC20AbiDecoder};

use super::error;

// use super::FSTK_TOKEN_TRANSFER_REQUEST_DISPATCHER_INTERFACE;
use super::{ERC1376_TOKEN_INTERFACE, ERC20_TOKEN_INTERFACE};
