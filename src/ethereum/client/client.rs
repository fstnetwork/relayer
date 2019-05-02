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
use std::fmt;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};

use bytes::Bytes;
use ethcore::transaction::SignedTransaction;
use ethereum_types::{Address, H256, U256};
use futures::{Future, Stream};
use hyper::client::HttpConnector;
use hyper::{Body, Client, Request, Uri};
use jsonrpc_core::request::MethodCall;
use jsonrpc_core::response::{
    Failure as JsonRpcFailure, Output as JsonRpcOutput, Success as JsonRpcSuccess,
};
use jsonrpc_core::{Id, Params, Version};
use rlp::{Encodable, RlpStream};
use rustc_hex::ToHex;
use serde_json::{self as json, Value as JsonValue};

use crate::contract_abi::{Erc1376AbiDecoder, Erc1376AbiEncoder, Erc20AbiDecoder, Erc20AbiEncoder};
use crate::types::{AccountState, BlockId, EthTransactionConfirmation};
use crate::types::{EthRpcBytes, EthRpcCallRequest, EthRpcTransaction, EthRpcTransactionReceipt};
use crate::utils::clean_0x;

use super::error::{Error, ErrorKind};

pub struct JsonRpcClient {
    host: Uri,
    client: Client<HttpConnector, Body>,
    counter: AtomicUsize,
}

impl JsonRpcClient {
    pub fn new(host: String) -> JsonRpcClient {
        let host = Uri::from_shared(Bytes::from(host.clone())).unwrap();
        let client = Client::builder().keep_alive(true).build_http();
        JsonRpcClient {
            host,
            client,
            counter: AtomicUsize::default(),
        }
    }

    pub fn request(
        &self,
        method: &'static str,
        params: Vec<JsonValue>,
    ) -> impl Future<Item = JsonRpcOutput, Error = Error> {
        let id = self.counter.fetch_add(1, Ordering::Relaxed);
        let method_call = MethodCall {
            jsonrpc: Some(Version::V2),
            method: method.to_owned(),
            params: Params::Array(params),
            id: Id::Num(id as u64),
        };

        let serialized = json::to_string(&method_call).expect("request is serializable; qed");
        let request = Request::post(&self.host)
            .header("Content-Type", "application/json")
            .body(serialized.into())
            .unwrap();

        self.client
            .request(request)
            .and_then(|res| res.into_body().concat2())
            .from_err::<Error>()
            .and_then(|data| Ok(json::from_slice::<JsonRpcOutput>(&data)?))
            .from_err()
    }

    pub fn batch_request(
        &self,
        batch: &Vec<(String, Vec<JsonValue>)>,
    ) -> impl Future<Item = Vec<Result<JsonValue, Error>>, Error = Error> {
        use jsonrpc_core::types::request::{Call, Request as JsonRpcRequest};

        let requests = JsonRpcRequest::Batch(
            batch
                .iter()
                .map(|(method, params)| {
                    let id = self.counter.fetch_add(1, Ordering::Relaxed);
                    Call::MethodCall(MethodCall {
                        jsonrpc: Some(Version::V2),
                        method: method.to_owned(),
                        params: Params::Array(params.clone()),
                        id: Id::Num(id as u64),
                    })
                })
                .collect(),
        );

        let serialized = json::to_string(&requests).expect("request is serializable; qed");
        let request = Request::post(&self.host)
            .header("Content-Type", "application/json")
            .body(serialized.into())
            .unwrap();

        self.client
            .request(request)
            .and_then(|res| res.into_body().concat2())
            .from_err::<Error>()
            .and_then(|data| {
                let res = json::from_slice::<Vec<JsonRpcOutput>>(&data)?
                    .into_iter()
                    .map(extract_result)
                    .collect();
                Ok(res)
            })
            .from_err()
    }
}

pub struct EthereumClient {
    client: JsonRpcClient,
}

impl EthereumClient {
    pub fn new(host: String) -> EthereumClient {
        EthereumClient {
            client: JsonRpcClient::new(host),
        }
    }

    #[inline]
    pub fn request(
        &self,
        method: &'static str,
        params: Vec<JsonValue>,
    ) -> impl Future<Item = JsonRpcOutput, Error = Error> {
        self.client.request(method, params)
    }

    #[inline]
    pub fn batch_request(
        &self,
        batch: &Vec<(String, Vec<JsonValue>)>,
    ) -> impl Future<Item = Vec<Result<JsonValue, Error>>, Error = Error> {
        self.client.batch_request(batch)
    }

    pub fn eth_call(&self, params: JsonValue) -> impl Future<Item = JsonValue, Error = Error> {
        self.request("eth_call", vec![params])
            .and_then(extract_result)
    }

    pub fn eth_read_contract(
        &self,
        contract_address: &Address,
        data: &Vec<u8>,
    ) -> impl Future<Item = Vec<u8>, Error = Error> {
        let params = json!({
           "to": to_0xhex(contract_address),
            "data": bytes_to_0xhex(data),
        });

        self.eth_call(params).and_then(to_bytes)
    }

    /// returns Some(RpcTransaction) if transaction exists
    /// returns None if transaction does not exist
    pub fn eth_get_transaction_by_hash(
        &self,
        tx_hash: &H256,
    ) -> impl Future<Item = Option<EthRpcTransaction>, Error = Error> {
        self.request("eth_getTransactionByHash", vec![to_0xhex(tx_hash).into()])
            .and_then(extract_transaction)
    }

    pub fn eth_get_transaction_receipt(
        &self,
        tx_hash: &H256,
    ) -> impl Future<Item = Option<EthRpcTransactionReceipt>, Error = Error> {
        self.request("eth_getTransactionReceipt", vec![to_0xhex(tx_hash).into()])
            .and_then(extract_transaction_receipt)
    }

    pub fn eth_get_transaction_confirmation(
        &self,
        tx_hash: &H256,
    ) -> impl Future<Item = EthTransactionConfirmation, Error = Error> {
        self.batch_request(&vec![
            ("eth_blockNumber".into(), vec![]),
            (
                "eth_getTransactionReceipt".into(),
                vec![to_0xhex(tx_hash).into()],
            ),
        ])
        .and_then(extract_transaction_confirmation)
    }

    pub fn eth_estimate_gas(
        &self,
        call_request: EthRpcCallRequest,
    ) -> impl Future<Item = U256, Error = Error> {
        let call_request =
            json::to_value(call_request).expect("EthRpcCallRequest is serializable; qed");
        self.request("eth_estimateGas", vec![call_request])
            .and_then(extract_hex_value)
    }

    pub fn eth_get_block_gas_limit(
        &self,
        block_id: BlockId,
    ) -> impl Future<Item = U256, Error = Error> {
        let block_id = block_id.to_string();

        self.request("eth_getBlockByNumber", vec![block_id.into(), true.into()])
            .and_then(|value: JsonRpcOutput| match value {
                JsonRpcOutput::Success(JsonRpcSuccess { result, .. }) => {
                    if let JsonValue::Object(block) = result {
                        if let Some(gas_limit) = block["gasLimit"].as_str() {
                            match U256::from_str(clean_0x(&gas_limit)) {
                                Ok(v) => Ok(v),
                                Err(_) => Err(Error::from(ErrorKind::ParseHex)),
                            }
                        } else {
                            Err(Error::from(ErrorKind::NoSuchField("gasLimit")))
                        }
                    } else {
                        // FIXME better error message
                        Err(Error::from(ErrorKind::NoSuchField("block")))
                    }
                }
                JsonRpcOutput::Failure(JsonRpcFailure { error, .. }) => {
                    Err(Error::from(ErrorKind::JsonRpc(error.clone())))
                }
            })
    }

    pub fn eth_block_number(&self) -> impl Future<Item = U256, Error = Error> {
        self.request("eth_blockNumber", vec![])
            .and_then(extract_hex_value)
    }

    pub fn eth_balance_of(&self, address: &Address) -> impl Future<Item = U256, Error = Error> {
        self.request("eth_getBalance", vec![to_0xhex(address).into()])
            .and_then(extract_hex_value)
    }

    pub fn eth_nonce_of(&self, address: &Address) -> impl Future<Item = U256, Error = Error> {
        self.request("eth_getTransactionCount", vec![to_0xhex(address).into()])
            .and_then(extract_hex_value)
    }

    pub fn eth_code_of(&self, address: &Address) -> impl Future<Item = Vec<u8>, Error = Error> {
        self.request("eth_getCode", vec![to_0xhex(address).into()])
            .and_then(extract_bytes)
    }

    pub fn eth_send_raw_transaction(
        &self,
        tx: &SignedTransaction,
    ) -> impl Future<Item = H256, Error = Error> {
        let mut stream = RlpStream::new();
        tx.rlp_append(&mut stream);
        let hex = stream.out().to_hex();

        self.request("eth_sendRawTransaction", vec![format!("0x{}", hex).into()])
            .and_then(extract_hex_value)
    }

    pub fn eth_state_of(
        &self,
        address: &Address,
    ) -> impl Future<Item = AccountState, Error = Error> {
        self.batch_request({
            let address_param: jsonrpc_core::Value = to_0xhex(address).into();
            &vec![
                (
                    "eth_getTransactionCount".into(),
                    vec![address_param.clone()],
                ),
                ("eth_getBalance".into(), vec![address_param]),
            ]
        })
        .and_then({
            let address = address.clone();
            move |results| extract_nonce_and_balance(address, results)
        })
    }

    pub fn token_symbol(
        &self,
        token_address: &Address,
    ) -> impl Future<Item = String, Error = Error> {
        self.eth_read_contract(token_address, &Erc20AbiEncoder::symbol())
            .and_then(|data| Ok(Erc20AbiDecoder::symbol(&data)?))
            .from_err()
    }

    pub fn token_delegate_enable(
        &self,
        token_address: &Address,
    ) -> impl Future<Item = bool, Error = Error> {
        self.eth_read_contract(token_address, &Erc1376AbiEncoder::is_delegate_enable())
            .and_then(|data| Ok(Erc1376AbiDecoder::is_delegate_enable(&data)?))
            .from_err()
    }

    pub fn token_balance_of(
        &self,
        token_address: &Address,
        address: &Address,
    ) -> impl Future<Item = U256, Error = Error> {
        self.eth_read_contract(token_address, &Erc20AbiEncoder::balance_of(address))
            .and_then(|data| Ok(Erc20AbiDecoder::balance_of(&data)?))
            .from_err()
    }

    pub fn token_nonce_of(
        &self,
        token_address: &Address,
        address: &Address,
    ) -> impl Future<Item = U256, Error = Error> {
        self.eth_read_contract(token_address, &Erc1376AbiEncoder::nonce_of(address))
            .and_then(|data| Ok(Erc1376AbiDecoder::nonce_of(&data)?))
            .from_err()
    }

    pub fn token_state_of(
        &self,
        token_address: &Address,
        address: &Address,
    ) -> impl Future<Item = AccountState, Error = Error> {
        let to: serde_json::Value = to_0xhex(token_address).into();
        let nonce_call = json!({
                        "to": to.clone(),
                        "data": bytes_to_0xhex(&Erc1376AbiEncoder::nonce_of(address)),
        });
        let balance_call = json!({
            "to": to,
            "data": bytes_to_0xhex(&Erc20AbiEncoder::balance_of(address)),
        });

        self.batch_request({
            &vec![
                ("eth_call".into(), vec![nonce_call]),
                ("eth_call".into(), vec![balance_call]),
            ]
        })
        .and_then({
            let address = address.clone();
            move |results| extract_nonce_and_balance(address, results)
        })
    }
}

pub fn extract_result(value: JsonRpcOutput) -> Result<JsonValue, Error> {
    match value {
        JsonRpcOutput::Success(JsonRpcSuccess { result, .. }) => Ok(result),
        JsonRpcOutput::Failure(JsonRpcFailure { error, .. }) => {
            Err(Error::from(ErrorKind::JsonRpc(error.clone())))
        }
    }
}

pub fn extract_bytes(value: JsonRpcOutput) -> Result<Vec<u8>, Error> {
    extract_result(value).and_then(to_bytes)
}

pub fn extract_transaction(value: JsonRpcOutput) -> Result<Option<EthRpcTransaction>, Error> {
    extract_result(value).and_then(|value| match value.is_object() {
        true => Ok(Some(json::from_value::<EthRpcTransaction>(value)?)),
        false => Ok(None),
    })
}

pub fn extract_transaction_receipt(
    value: JsonRpcOutput,
) -> Result<Option<EthRpcTransactionReceipt>, Error> {
    extract_result(value).and_then(|value| match value.is_object() {
        true => Ok(Some(json::from_value::<EthRpcTransactionReceipt>(value)?)),
        false => Ok(None),
    })
}

pub fn extract_nonce_and_balance(
    address: Address,
    mut results: Vec<Result<JsonValue, Error>>,
) -> Result<AccountState, Error> {
    // vec[nonce, balance]

    if results.len() != 2 {
        return Err(Error::from(format!(
            "expected length of result is 2, but got {:?}",
            results.len()
        )));
    }

    let hex_value = |results: &mut Vec<Result<JsonValue, Error>>| {
        let result = match results.pop() {
            Some(Ok(value)) => value,
            Some(Err(e)) => return Err(e),
            None => return Err(Error::from("result is empty")),
        };

        U256::from_str(clean_0x(&serde_json::from_value::<String>(result)?))
            .map_err(|_| Error::from(ErrorKind::ParseHex))
    };

    let balance = hex_value(&mut results)?;
    let nonce = hex_value(&mut results)?;

    Ok(AccountState::new(address, nonce, balance))
}

pub fn extract_transaction_confirmation(
    mut results: Vec<Result<JsonValue, Error>>,
) -> Result<EthTransactionConfirmation, Error> {
    // vec[block_number, receipt]

    if results.len() != 2 {
        return Err(Error::from(format!(
            "expected length of result is 2, but got {:?}",
            results.len()
        )));
    }

    let receipt: Option<EthRpcTransactionReceipt> =
        match results.pop().expect("result is not empty; qed") {
            Ok(r) => json::from_value(r)?,
            Err(_err) => None,
        };

    let block_number = {
        let value = match results.pop().expect("result is not empty; qed") {
            Ok(n) => n,
            Err(err) => return Err(err),
        };

        match U256::from_str(clean_0x(&serde_json::from_value::<String>(value)?)) {
            Ok(n) => n,
            Err(_) => return Err(Error::from(ErrorKind::ParseHex)),
        }
    };

    Ok(EthTransactionConfirmation {
        receipt,
        block_number,
    })
}

pub fn extract_hex_value<T>(value: JsonRpcOutput) -> Result<T, Error>
where
    T: FromStr,
{
    extract_result(value).and_then(|value| {
        T::from_str(clean_0x(&json::from_value::<String>(value)?))
            .map_err(|_| Error::from(ErrorKind::ParseHex))
    })
}

pub fn to_bytes(value: JsonValue) -> Result<Vec<u8>, Error> {
    Ok(json::from_value::<EthRpcBytes>(value)?.into())
}

#[inline]
pub fn to_0xhex(value: &fmt::LowerHex) -> String {
    format!("0x{:x}", value)
}

pub fn bytes_to_0xhex(bytes: &Vec<u8>) -> String {
    format!("0x{}", bytes.as_slice().to_hex())
}
