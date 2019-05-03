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
use std::collections::HashMap;
use std::sync::Arc;

use super::error::Error;
use super::ethereum_client::EthereumClient;

pub struct ClientGroup {
    clients: HashMap<String, Arc<EthereumClient>>,
}

impl ClientGroup {
    pub fn new(ethereum_nodes: Vec<String>) -> ClientGroup {
        let clients = ethereum_nodes
            .iter()
            .fold(HashMap::new(), |mut clients, host| {
                clients.insert(host.clone(), Arc::new(EthereumClient::new(host.clone())));
                clients
            });

        ClientGroup { clients }
    }

    pub fn pick(&self) -> Result<Arc<EthereumClient>, Error> {
        if self.clients.is_empty() {
            return Err(Error::EthereumClientGroupEmpty);
        }

        // TODO better implementation
        let client = self.clients.values().next().unwrap().clone();
        Ok(client)
    }

    pub fn add(&mut self, endpoint: String) -> bool {
        self.clients
            .insert(endpoint.clone(), Arc::new(EthereumClient::new(endpoint)))
            .is_some()
    }

    pub fn remove(&mut self, endpoint: &String) -> bool {
        self.clients.remove(endpoint).is_some()
    }

    pub fn contains(&mut self, endpoint: &String) -> bool {
        self.clients.contains_key(endpoint)
    }

    pub fn set_endpoints(&mut self, endpoints: Vec<String>) {
        for endpoint in endpoints.iter() {
            if !self.contains(endpoint) {
                self.add(endpoint.clone());
            }
        }

        for endpoint in self.endpoints().iter() {
            if !endpoints.contains(endpoint) {
                self.remove(endpoint);
            }
        }
    }

    pub fn endpoints(&self) -> Vec<String> {
        self.clients.keys().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.clients.len()
    }
}
