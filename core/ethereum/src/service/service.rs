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
use std::sync::Arc;

use ethereum_types::{Address, H256, U256};
use futures::{self, future, Future};

use contract_abi::ERC1376AbiEncoder;
use traits::{
    AccountStateFuture, BoolFuture, BytesFuture, EthRpcTransactionConfirmationFuture,
    EthRpcTransactionFuture, EthRpcTransactionReceiptFuture, H256Future, U256Future,
};
use types::{BlockId, Currency, EthRpcCallRequest, GasEstimation};

use super::error::Error;

use super::client_group::ClientGroup;
use super::ethereum_client::EthereumClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Params {
    pub ethereum_nodes: Vec<String>,
}

pub struct Service {
    client_group: ClientGroup,
}

impl traits::EthereumService for Service {
    type Error = Error;

    fn add_endpoint(&mut self, endpoint: String) -> bool {
        self.client_group.add(endpoint)
    }

    fn remove_endpoint(&mut self, endpoint: &String) -> bool {
        self.client_group.remove(endpoint)
    }

    fn contains_endpoint(&mut self, endpoint: &String) -> bool {
        self.client_group.contains(endpoint)
    }

    fn endpoints(&self) -> Vec<String> {
        self.client_group.endpoints()
    }

    fn endpoint_count(&self) -> usize {
        self.client_group.len()
    }
}

impl Service {
    pub fn new(params: Params) -> Result<Service, Error> {
        let client_group = ClientGroup::new(params.ethereum_nodes);

        Ok(Service { client_group })
    }

    #[inline]
    fn pick_client(&self) -> Result<Arc<EthereumClient>, Error> {
        self.client_group.pick()
    }

    #[inline]
    fn pick_client_future(&self) -> future::FutureResult<Arc<EthereumClient>, Error> {
        futures::done(self.pick_client())
    }
}

impl traits::GasEstimator<<Service as traits::EthereumService>::Error> for Service {
    fn block_gas_limit(&self, block_id: BlockId) -> U256Future<Error> {
        let client = self.pick_client_future();
        Box::new(client.and_then(move |client| client.eth_get_block_gas_limit(block_id).from_err()))
    }

    fn estimate_gas(
        &self,
        gas_estimation: GasEstimation,
    ) -> U256Future<<Service as traits::EthereumService>::Error> {
        let call = match gas_estimation {
            GasEstimation::Transaction(tx) => {
                let receiver = match tx.action {
                    ethcore_transaction::Action::Call(address) => address,
                    ethcore_transaction::Action::Create => Address::new(),
                };
                EthRpcCallRequest {
                    to: Some(receiver.into()),
                    from: Some(tx.sender().into()),
                    gas: None,
                    gas_price: None,
                    data: Some(tx.data.clone().into()),
                    value: Some(tx.value.into()),
                }
            }
            GasEstimation::TokenTransferRequest {
                relayer_address,
                signed_request,
            } => {
                let token_address = signed_request.unverified().token().clone();

                EthRpcCallRequest {
                    to: Some(token_address.into()),
                    from: Some(relayer_address.into()),
                    gas: None,
                    gas_price: None,
                    data: Some(
                        ERC1376AbiEncoder::delegate_transfer_and_call(&signed_request).into(),
                    ),
                    value: Some(U256::from(0).into()),
                }
            }
        };

        let client = self.pick_client_future();
        Box::new(client.and_then(move |client| client.eth_estimate_gas(call).from_err()))
    }
}

impl traits::AccountStateProvider<<Service as traits::EthereumService>::Error> for Service {
    fn balance_of(
        &self,
        address: Address,
        currency: Currency,
    ) -> U256Future<<Service as traits::EthereumService>::Error> {
        let client = self.pick_client_future();
        match currency {
            Currency::Ether => {
                Box::new(client.and_then(move |client| client.eth_balance_of(&address).from_err()))
            }
            Currency::Token(token_address) => Box::new(client.and_then(move |client| {
                client.token_balance_of(&token_address, &address).from_err()
            })),
        }
    }

    fn nonce_of(
        &self,
        address: Address,
        currency: Currency,
    ) -> U256Future<<Service as traits::EthereumService>::Error> {
        let client_future = self.pick_client_future();
        match currency {
            Currency::Ether => Box::new(
                client_future.and_then(move |client| client.eth_nonce_of(&address).from_err()),
            ),
            Currency::Token(token_address) => Box::new(client_future.and_then(move |client| {
                client.token_nonce_of(&token_address, &address).from_err()
            })),
        }
    }

    fn state_of(
        &self,
        address: Address,
        currency: Currency,
    ) -> AccountStateFuture<<Service as traits::EthereumService>::Error> {
        let client_future = self.pick_client_future();
        match currency {
            Currency::Ether => Box::new(
                client_future.and_then(move |client| client.eth_state_of(&address).from_err()),
            ),
            Currency::Token(token_address) => Box::new(client_future.and_then(move |client| {
                client.token_state_of(&token_address, &address).from_err()
            })),
        }
    }

    fn code_of(
        &self,
        address: Address,
    ) -> BytesFuture<<Service as traits::EthereumService>::Error> {
        Box::new(
            self.pick_client_future()
                .and_then(move |client| client.eth_code_of(&address).from_err()),
        )
    }
}

impl traits::TokenStateProvider<<Service as traits::EthereumService>::Error> for Service {
    fn token_delegate_enable(
        &self,
        token_contract: Address,
    ) -> BoolFuture<<Service as traits::EthereumService>::Error> {
        Box::new(
            self.pick_client_future()
                .and_then(move |client| client.token_delegate_enable(&token_contract).from_err()),
        )
    }
}

impl traits::BlockInfoProvider<<Service as traits::EthereumService>::Error> for Service {
    fn get_block_number(&self) -> U256Future<<Service as traits::EthereumService>::Error> {
        Box::new(
            self.pick_client_future()
                .and_then(move |client| client.eth_block_number().from_err()),
        )
    }
}

impl traits::TransactionFetcher<<Service as traits::EthereumService>::Error> for Service {
    fn get_transaction_by_hash(
        &self,
        tx_hash: H256,
    ) -> EthRpcTransactionFuture<<Service as traits::EthereumService>::Error> {
        Box::new(
            self.pick_client_future()
                .and_then(move |client| client.eth_get_transaction_by_hash(&tx_hash).from_err()),
        )
    }

    fn get_transaction_receipt(
        &self,
        tx_hash: H256,
    ) -> EthRpcTransactionReceiptFuture<<Service as traits::EthereumService>::Error> {
        Box::new(
            self.pick_client_future()
                .and_then(move |client| client.eth_get_transaction_receipt(&tx_hash).from_err()),
        )
    }

    fn get_transaction_confirmation(
        &self,
        tx_hash: H256,
    ) -> EthRpcTransactionConfirmationFuture<<Service as traits::EthereumService>::Error> {
        Box::new(
            self.pick_client_future().and_then(move |client| {
                client.eth_get_transaction_confirmation(&tx_hash).from_err()
            }),
        )
    }
}

impl traits::TransactionBroadcaster<<Service as traits::EthereumService>::Error> for Service {
    fn send_transaction(
        &self,
        tx: ethcore_transaction::SignedTransaction,
    ) -> H256Future<<Service as traits::EthereumService>::Error> {
        Box::new(
            self.pick_client_future()
                .and_then(move |client| client.eth_send_raw_transaction(&tx).from_err()),
        )
    }
}
