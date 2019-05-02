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
use super::{Log, H160, H2048, H256, U256, U64};

/// Transaction Receipt
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransactionReceipt {
    /// Transaction Hash
    #[serde(rename = "transactionHash")]
    pub transaction_hash: Option<H256>,

    /// Transaction index
    #[serde(rename = "transactionIndex")]
    pub transaction_index: Option<U256>,

    /// Block hash
    #[serde(rename = "blockHash")]
    pub block_hash: Option<H256>,

    /// Sender
    pub from: Option<H160>,

    /// Recipient
    pub to: Option<H160>,

    /// Block number
    #[serde(rename = "blockNumber")]
    pub block_number: Option<U256>,

    /// Cumulative gas used
    #[serde(rename = "cumulativeGasUsed")]
    pub cumulative_gas_used: U256,

    /// Gas used
    #[serde(rename = "gasUsed")]
    pub gas_used: Option<U256>,

    /// Contract address
    #[serde(rename = "contractAddress")]
    pub contract_address: Option<H160>,

    /// Logs
    pub logs: Vec<Log>,

    /// State Root
    #[serde(rename = "root")]
    pub state_root: Option<H256>,

    /// Logs bloom
    #[serde(rename = "logsBloom")]
    pub logs_bloom: H2048,

    /// Status code
    #[serde(rename = "status")]
    pub status_code: Option<U64>,
}
