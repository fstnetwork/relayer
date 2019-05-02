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
use ethereum_types::Address;
use std::collections::HashSet;

pub trait AddressFilter: Clone {
    fn is_allowed(&self, address: &Address) -> bool;

    fn is_denied(&self, address: &Address) -> bool {
        !self.is_allowed(address)
    }
}

#[derive(Clone)]
pub struct DummyAddressFilter();

impl DummyAddressFilter {
    fn new() -> DummyAddressFilter {
        DummyAddressFilter {}
    }
}

impl AddressFilter for DummyAddressFilter {
    fn is_allowed(&self, _token: &Address) -> bool {
        true
    }
}

#[derive(Copy, Clone)]
pub enum ListAddressFilterMode {
    Blacklist,
    Whitelist,
}

#[derive(Clone)]
pub struct ListAddressFilter {
    list: HashSet<Address>,
    mode: ListAddressFilterMode,
}

impl ListAddressFilter {
    pub fn new(mode: ListAddressFilterMode) -> ListAddressFilter {
        ListAddressFilter {
            mode,
            list: Default::default(),
        }
    }

    pub fn with_list(mode: ListAddressFilterMode, allow_tokens: Vec<Address>) -> ListAddressFilter {
        ListAddressFilter {
            mode,
            list: allow_tokens
                .into_iter()
                .fold(HashSet::default(), |mut m, token| {
                    m.insert(token);
                    m
                }),
        }
    }

    pub fn add_token(&mut self, token: Address) {
        self.list.insert(token);
    }

    pub fn remove_token(&mut self, token: &Address) {
        self.list.remove(token);
    }

    pub fn set_mode(&mut self, mode: ListAddressFilterMode) {
        self.mode = mode;
    }

    pub fn mode(&self) -> ListAddressFilterMode {
        self.mode
    }

    pub fn tokens(&self) -> &HashSet<Address> {
        &self.list
    }
}

impl Default for ListAddressFilter {
    fn default() -> ListAddressFilter {
        ListAddressFilter {
            list: HashSet::new(),
            mode: ListAddressFilterMode::Blacklist,
        }
    }
}

impl AddressFilter for ListAddressFilter {
    fn is_allowed(&self, address: &Address) -> bool {
        let contains = self.list.contains(address);

        match self.mode {
            ListAddressFilterMode::Whitelist => contains,
            ListAddressFilterMode::Blacklist => !contains,
        }
    }
}
