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
use ethereum_types::{Address, U256};
use std::collections::HashMap;
use std::{cmp, fmt};

use types::DelegateMode;

use super::{Change, Choice, PoolRequest};

pub trait RequestSelector<R>: Sync + Send + fmt::Debug {
    type Score: Sync + Send + Clone + cmp::Ord + Default + fmt::Debug + fmt::LowerHex;
    type Event: fmt::Debug;

    fn compare(&self, old: &R, other: &R) -> cmp::Ordering;

    fn choose(&self, old: &R, new: &R) -> Choice;

    fn update_scores(&self, txs: &[R], scores: &mut [Self::Score], change: Change<Self::Event>);

    fn should_replace(&self, old: &R, new: &R) -> Choice;

    fn should_ignore_sender_limit(&self, _new: &R) -> bool {
        false
    }
}

#[derive(Debug, Default)]
pub struct NonceAndFeeSelector {}

impl NonceAndFeeSelector {
    fn delegate_mode_priority(mode: DelegateMode) -> u8 {
        let mode: u8 = mode.into();
        u8::max_value() - mode
    }
}

impl<R> RequestSelector<R> for NonceAndFeeSelector
where
    R: PoolRequest,
{
    type Score = U256;
    type Event = R;

    fn compare(&self, old: &R, other: &R) -> cmp::Ordering {
        old.nonce().cmp(&other.nonce())
    }

    fn choose(&self, old: &R, new: &R) -> Choice {
        if old.nonce() != new.nonce() {
            return Choice::InsertNew;
        }

        match new.fee().cmp(old.fee()) {
            cmp::Ordering::Greater => Choice::ReplaceOld,
            _ => Choice::RejectNew,
        }
    }

    fn should_replace(&self, old: &R, new: &R) -> Choice {
        if old.sender() == new.sender() {
            // prefer earliest request
            match new.nonce().cmp(&old.nonce()) {
                cmp::Ordering::Less => Choice::ReplaceOld,
                cmp::Ordering::Greater => Choice::RejectNew,
                cmp::Ordering::Equal => self.choose(old, new),
            }
        } else {
            let old_score = (Self::delegate_mode_priority(old.delegate_mode()), old.fee());
            let new_score = (Self::delegate_mode_priority(new.delegate_mode()), new.fee());
            if new_score > old_score {
                Choice::ReplaceOld
            } else {
                Choice::RejectNew
            }
        }
    }

    fn update_scores(
        &self,
        requests: &[R],
        scores: &mut [Self::Score],
        change: Change<Self::Event>,
    ) {
        match change {
            Change::Culled(_) => {}
            Change::RemovedAt(_) => {}
            Change::InsertedAt(i) | Change::ReplacedAt(i) => {
                assert!(i < requests.len());
                assert!(i < scores.len());

                scores[i] = *requests[i].fee();
                // TODO
                // let boost = match requests[i].priority() {
                //     super::Priority::Local => 15,
                //     super::Priority::Retracted => 10,
                //     super::Priority::Regular => 0,
                // };
                // scores[i] = scores[i] << boost;
            }
            // We are only sending an event in case of penalization.
            // So just lower the priority of all non-local transactions.
            Change::Event(_) => {
                // TODO
                // for (score, req) in scores.iter_mut().zip(requests) {
                // Never penalize local transactions.
                // if !req.priority().is_local() {
                //     *score = *score >> 3;
                // }
                // }
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct TokenSelector {
    base_selector: NonceAndFeeSelector,
    token_priorities: HashMap<Address, u32>,
}

impl TokenSelector {
    pub fn new() -> TokenSelector {
        TokenSelector::default()
    }

    pub fn with_priorities(token_priorities: HashMap<Address, u32>) -> TokenSelector {
        TokenSelector {
            base_selector: NonceAndFeeSelector::default(),
            token_priorities,
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.token_priorities.clear();
    }

    #[inline]
    pub fn token_priority(&self, token: &Address) -> u32 {
        *self.token_priorities.get(token).unwrap_or(&0)
    }

    #[inline]
    pub fn token_priorities(&self) -> &HashMap<Address, u32> {
        &self.token_priorities
    }

    #[inline]
    pub fn tokens(&self) -> Vec<Address> {
        self.token_priorities.keys().cloned().collect()
    }

    #[inline]
    pub fn add_token_priority(&mut self, token: Address, priority: u32) {
        self.token_priorities.insert(token, priority);
    }

    #[inline]
    pub fn remove_token_priorities(&mut self, token: &Address) -> u32 {
        self.token_priorities.remove(token).unwrap_or(0)
    }

    #[inline]
    pub fn priority_pair<R: PoolRequest>(&self, old: &R, new: &R) -> (u32, u32) {
        (
            self.token_priority(old.token()),
            self.token_priority(new.token()),
        )
    }
}

impl<R> RequestSelector<R> for TokenSelector
where
    R: PoolRequest,
{
    type Score = U256;
    type Event = R;

    fn compare(&self, old: &R, other: &R) -> cmp::Ordering {
        if old.token() != other.token() {
            let (old, other) = self.priority_pair(old, other);
            return old.cmp(&other);
        }

        self.base_selector.compare(old, other)
    }

    fn choose(&self, old: &R, new: &R) -> Choice {
        if old.token() != new.token() {
            return Choice::InsertNew;
        }

        self.base_selector.choose(old, new)
    }

    fn should_replace(&self, old: &R, new: &R) -> Choice {
        if old.token() != new.token() {
            let (old_prio, new_prio) = self.priority_pair(old, new);
            return match new_prio.cmp(&old_prio) {
                cmp::Ordering::Greater => Choice::ReplaceOld,
                cmp::Ordering::Equal => self.base_selector.choose(old, new),
                cmp::Ordering::Less => Choice::RejectNew,
            };
        }

        self.base_selector.should_replace(old, new)
    }

    fn update_scores(
        &self,
        requests: &[R],
        scores: &mut [Self::Score],
        change: Change<Self::Event>,
    ) {
        match change {
            Change::Culled(_) => {}
            Change::RemovedAt(_) => {}
            Change::InsertedAt(i) | Change::ReplacedAt(i) => {
                assert!(i < requests.len());
                assert!(i < scores.len());

                scores[i] = self.token_priority(requests[i].token()).into();
                self.base_selector.update_scores(requests, scores, change);
            }
            Change::Event(_) => {
                for (score, req) in scores.iter_mut().zip(requests) {
                    *score = self.token_priority(req.token()).into();
                }
                self.base_selector.update_scores(requests, scores, change);
            }
        }
    }
}
