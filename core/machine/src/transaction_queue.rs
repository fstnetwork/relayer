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
use std::collections::{HashMap, HashSet};

use ethereum_types::{Address, H256, U256};

use ethcore_transaction::SignedTransaction;

pub struct TransactionQueue {
    inner: HashMap<Address, TransactionQueueInner>,
}

struct TransactionQueueInner {
    queue: Vec<SignedTransaction>,
    transactions_set: HashSet<H256>,
    current_nonce: U256,
}

impl TransactionQueue {
    pub fn new() -> TransactionQueue {
        TransactionQueue {
            inner: HashMap::new(),
        }
    }
}

impl TransactionQueueInner {
    pub fn new() -> TransactionQueueInner {
        TransactionQueueInner {
            queue: Vec::new(),
            transactions_set: HashSet::new(),
            current_nonce: 0.into(),
        }
    }

    pub fn current_nonce(&self) -> U256 {
        self.current_nonce
    }

    pub fn push(&mut self, tx: SignedTransaction) {
        if tx.as_unsigned().nonce < self.current_nonce {
            return;
        }

        let hash = tx.hash();
        if !self.transactions_set.contains(&hash) {
            self.current_nonce = tx.as_unsigned().nonce;
            self.transactions_set.insert(hash);
            self.queue.push(tx);
        }
    }

    pub fn remove_by_nonce(&mut self, nonce: &U256) {
        if *nonce > self.current_nonce {
            return;
        }

        let mut hash = H256::default();
        for tx in &self.queue {
            if tx.as_unsigned().nonce == *nonce {
                hash = tx.hash();
                break;
            }
        }

        self.transactions_set.remove(&hash);
        self.queue.retain(|tx| tx.as_unsigned().nonce != *nonce);
    }

    pub fn remove_by_hash(&mut self, hash: &H256) {
        if !self.transactions_set.contains(hash) {
            return;
        }

        self.transactions_set.remove(hash);
        self.queue.retain(|tx| tx.hash() != *hash);
    }
}
