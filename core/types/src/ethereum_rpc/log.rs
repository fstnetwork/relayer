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
use super::{Bytes, H160, H256, U256};

/// Log
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Clone)]
pub struct Log {
    /// H160
    pub address: H160,

    /// Topics
    pub topics: Vec<H256>,

    /// Data
    pub data: Bytes,

    /// Block Hash
    #[serde(rename = "blockHash")]
    pub block_hash: Option<H256>,

    /// Block Number
    #[serde(rename = "blockNumber")]
    pub block_number: Option<U256>,

    /// Transaction Hash
    #[serde(rename = "transactionHash")]
    pub transaction_hash: Option<H256>,

    /// Transaction Index
    #[serde(rename = "transactionIndex")]
    pub transaction_index: Option<U256>,

    /// Log Index in Block
    #[serde(rename = "logIndex")]
    pub log_index: Option<U256>,

    /// Log Index in Transaction
    #[serde(rename = "transactionLogIndex")]
    pub transaction_log_index: Option<U256>,

    /// Log Type
    #[serde(rename = "type")]
    pub log_type: String,

    /// Whether Log Type is Removed (Geth Compatibility Field)
    #[serde(default)]
    pub removed: bool,
}
