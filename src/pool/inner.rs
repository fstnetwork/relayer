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
use ethereum_types::{Address, H256, U256};
use std::collections::{hash_map, BTreeSet, HashMap, HashSet};
use std::slice;
use std::sync::Arc;

use crate::types::SignedRequest;

use super::{
    AddResult, Error, PoolParams, PoolRequest, PoolRequestTag, Readiness, ReadyChecker,
    RequestQueue, RequestSelector, ScoredRequest, Status,
};

pub struct InnerPool<R, S>
where
    R: PoolRequest,
    S: RequestSelector<R>,
{
    params: PoolParams,
    queues: HashMap<Address, RequestQueue<R, S>>,
    by_hash: HashMap<H256, R>,
    tags: HashMap<H256, PoolRequestTag>,
    best_requests: BTreeSet<ScoredRequest<S::Score, R>>,
    worst_requests: BTreeSet<ScoredRequest<S::Score, R>>,
    selector: Arc<S>,
    aggregate_gas: U256,
}

impl<R, S> InnerPool<R, S>
where
    R: PoolRequest,
    S: RequestSelector<R>,
{
    pub fn new(params: PoolParams, selector: Arc<S>) -> InnerPool<R, S> {
        InnerPool {
            params,
            queues: Default::default(),
            by_hash: Default::default(),
            tags: Default::default(),
            best_requests: Default::default(),
            worst_requests: Default::default(),
            selector,
            aggregate_gas: U256::zero(),
        }
    }

    #[inline]
    pub fn set_params(&mut self, params: PoolParams) {
        self.params = params;
    }

    #[inline]
    pub fn contains_hash(&self, hash: &H256) -> bool {
        self.by_hash.contains_key(hash)
    }

    pub fn import(&mut self, request: R) -> Result<Arc<SignedRequest>, Error> {
        if self.by_hash.contains_key(&request.hash()) {
            return Err(Error::AlreadyImported(format!("{:?}", request.hash())));
        }

        let (result, prev_state, current_state) = {
            let queue = self
                .queues
                .entry(*request.sender())
                .or_insert_with(RequestQueue::default);
            let prev = queue.worst_and_best();
            let result = queue.add(request.clone(), &self.selector, self.params.max_per_sender);
            let current = queue.worst_and_best();
            (result, prev, current)
        };

        self.update_senders_worst_and_best(prev_state, current_state);

        match result {
            AddResult::Ok(req) => {
                self.finalize_insert(&req, None);
                Ok(req.clone_signed())
            }
            AddResult::PushedOut { new, old } | AddResult::Replaced { new, old } => {
                self.finalize_insert(&new, Some(&old));
                Ok(new.clone_signed())
            }
            AddResult::TooCheap { new, old } => Err(Error::TooCheapToReplace {
                old_hash: format!("{:x}", old.hash()),
                hash: format!("{:x}", new.hash()),
            }),
            AddResult::TooCheapToEnter(new, score) => Err(Error::TooCheapToEnter {
                hash: format!("{:x}", new.hash()),
                min_score: format!("{:#x}", score),
            }),
        }
    }

    /// Updates best and worst request from a sender.
    fn update_senders_worst_and_best(
        &mut self,
        previous: Option<((S::Score, R), (S::Score, R))>,
        current: Option<((S::Score, R), (S::Score, R))>,
    ) {
        let worst_collection = &mut self.worst_requests;
        let best_collection = &mut self.best_requests;

        let is_same = |a: &(S::Score, R), b: &(S::Score, R)| a.0 == b.0 && a.1.hash() == b.1.hash();

        let update = |collection: &mut BTreeSet<_>, (score, req), remove| {
            if remove {
                collection.remove(&ScoredRequest::new(score, req));
            } else {
                collection.insert(ScoredRequest::new(score, req));
            }
        };

        match (previous, current) {
            (None, Some((worst, best))) => {
                update(worst_collection, worst, false);
                update(best_collection, best, false);
            }
            (Some((worst, best)), None) => {
                // all requests from that sender has been removed.
                // We can clear a hashmap entry.
                self.queues.remove(worst.1.sender());
                update(worst_collection, worst, true);
                update(best_collection, best, true);
            }
            (Some((w1, b1)), Some((w2, b2))) => {
                if !is_same(&w1, &w2) {
                    update(worst_collection, w1, true);
                    update(worst_collection, w2, false);
                }
                if !is_same(&b1, &b2) {
                    update(best_collection, b1, true);
                    update(best_collection, b2, false);
                }
            }
            (None, None) => {}
        }
    }

    #[inline]
    pub fn tags(&self) -> &HashMap<H256, PoolRequestTag> {
        &self.tags
    }

    #[inline]
    pub fn tags_mut(&mut self) -> &mut HashMap<H256, PoolRequestTag> {
        &mut self.tags
    }

    #[inline]
    pub fn get_tag(&self, hash: &H256) -> Option<&PoolRequestTag> {
        self.tags.get(hash)
    }

    #[inline]
    pub fn mark_by_hash(&mut self, hash: &H256, tag: PoolRequestTag) {
        self.tags.get_mut(hash).map(|v| *v = tag);
    }

    #[inline]
    pub fn mark_by_hashes(&mut self, hashes: &[H256], tag: PoolRequestTag) {
        hashes.iter().for_each(|hash| {
            self.tags.get_mut(hash).map(|v| *v = tag);
        });
    }

    pub fn remove(&mut self, hash: &H256) -> Option<Arc<SignedRequest>> {
        if let Some(req) = self.finalize_remove(hash) {
            self.remove_from_set(req.sender(), |set, selector| set.remove(&req, &selector));
            Some(req.clone_signed())
        } else {
            None
        }
    }

    pub fn remove_by_filter<F>(&mut self, filter: F) -> Vec<Arc<SignedRequest>>
    where
        F: FnMut(&R) -> bool,
    {
        let ready = |_req: &R| Readiness::Ready;
        let hashes: Vec<_> = self
            .unordered_pending(ready)
            .filter(filter)
            .map(|req| *req.hash())
            .collect();

        hashes.iter().fold(Vec::new(), |mut vec, hash| {
            if let Some(req) = self.remove(hash) {
                vec.push(req)
            }
            vec
        })
    }

    fn remove_stalled<Ready: ReadyChecker<R>>(
        &mut self,
        sender: &Address,
        ready: &mut Ready,
    ) -> usize {
        let removed_from_set =
            self.remove_from_set(sender, |queue, selector| queue.cull(ready, &selector));

        match removed_from_set {
            Some(removed) => {
                let len = removed.len();
                removed.iter().for_each(|req| {
                    self.finalize_remove(req.hash());
                });
                len
            }
            None => 0,
        }
    }

    /// Removes request from sender's request `HashMap`.
    fn remove_from_set<Res, F: FnOnce(&mut RequestQueue<R, S>, &S) -> Res>(
        &mut self,
        sender: &Address,
        f: F,
    ) -> Option<Res> {
        let (prev, next, result) = if let Some(set) = self.queues.get_mut(sender) {
            let prev = set.worst_and_best();
            let result = f(set, &self.selector);
            (prev, set.worst_and_best(), result)
        } else {
            return None;
        };

        self.update_senders_worst_and_best(prev, next);
        Some(result)
    }

    #[inline]
    pub fn remove_empty_queues(&mut self) {
        self.queues.retain(|_, queue| !queue.is_empty());
    }

    /// Removes all stalled requests from given sender list (or from all senders).
    #[allow(unused)]
    pub fn cull<Ready: ReadyChecker<R>>(
        &mut self,
        senders: Option<&[Address]>,
        mut ready: Ready,
    ) -> usize {
        let mut remove = |pool: &mut InnerPool<R, S>, senders: &[Address]| -> usize {
            senders.iter().fold(0, |removed, sender| {
                removed + pool.remove_stalled(sender, &mut ready)
            })
        };

        let removed = match senders {
            Some(senders) => remove(self, senders),
            None => {
                let senders = self.queues.keys().cloned().collect::<Vec<_>>();
                remove(self, &senders)
            }
        };

        self.remove_empty_queues();
        removed
    }

    fn finalize_insert(&mut self, request: &R, old: Option<&R>) {
        self.aggregate_gas += *request.gas_amount();
        self.by_hash.insert(request.hash().clone(), request.clone());
        self.tags
            .insert(request.hash().clone(), PoolRequestTag::Ready);

        if let Some(old) = old {
            self.finalize_remove(&old.hash());
        }
    }

    fn finalize_remove(&mut self, hash: &H256) -> Option<R> {
        self.tags.remove(hash);
        match self.by_hash.remove(hash) {
            Some(old) => {
                self.aggregate_gas -= *old.gas_amount();
                Some(old)
            }
            None => None,
        }
    }

    pub fn senders_with_token(&self) -> impl Iterator<Item = (&Address, HashSet<Address>)> {
        self.best_requests.iter().map(move |req| {
            let sender = req.request().sender();
            let tokens = self.tokens(sender).expect("sender is always existed; qed");
            (sender, tokens)
        })
    }

    #[inline]
    pub fn senders(&self) -> impl Iterator<Item = &Address> {
        self.best_requests.iter().map(|req| req.request().sender())
    }

    #[inline]
    pub fn tokens(&self, sender: &Address) -> Option<HashSet<Address>> {
        match self.queues.get(sender) {
            Some(queue) => Some(queue.tokens()),
            None => None,
        }
    }

    pub fn pending<Ready: ReadyChecker<R>>(&self, ready: Ready) -> PendingIterator<R, Ready, S> {
        PendingIterator {
            ready,
            best_requests: self.best_requests.clone(),
            pool: self,
        }
    }

    pub fn unordered_pending<Ready: ReadyChecker<R>>(
        &self,
        ready: Ready,
    ) -> UnorderedIterator<R, Ready, S> {
        UnorderedIterator {
            ready,
            senders: self.queues.iter(),
            queues: None,
        }
    }

    /// Update score of request of a particular sender.
    pub fn update_scores(&mut self, sender: &Address, event: S::Event) {
        let res = if let Some(set) = self.queues.get_mut(sender) {
            let prev = set.worst_and_best();
            set.update_scores(&self.selector, event);
            let current = set.worst_and_best();
            Some((prev, current))
        } else {
            None
        };

        if let Some((prev, current)) = res {
            self.update_senders_worst_and_best(prev, current);
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.queues.values().fold(0, |len, queue| len + queue.len())
    }

    #[inline]
    pub fn count_by_tag(&self, tag: PoolRequestTag) -> usize {
        self.tags
            .iter()
            .fold(0, |n, (_, v)| if tag.eq(v) { n + 1 } else { n })
    }

    pub fn clear(&mut self) {
        self.queues.clear();
        self.tags.clear();
        self.by_hash.clear();
        self.best_requests.clear();
        self.worst_requests.clear();
        self.aggregate_gas = U256::zero();
    }

    pub fn status(&self) -> HashMap<Address, Status> {
        // TODO
        unimplemented!();

        // let ready = |_req: &R| Readiness::Ready;
        // self.unordered_pending(ready).fold()
    }

    pub fn token_status(&self, _address: &Address) -> Status {
        // TODO
        unimplemented!();
    }
}

pub struct UnorderedIterator<'a, R, Ready, S>
where
    R: PoolRequest + 'a,
    Ready: ReadyChecker<R>,
    S: RequestSelector<R> + 'a,
{
    ready: Ready,
    senders: hash_map::Iter<'a, Address, RequestQueue<R, S>>,
    queues: Option<slice::Iter<'a, R>>,
}

impl<'a, R, Ready, S> Iterator for UnorderedIterator<'a, R, Ready, S>
where
    R: PoolRequest + 'a,
    Ready: ReadyChecker<R>,
    S: RequestSelector<R> + 'a,
{
    type Item = R;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(requests) = self.queues.as_mut() {
                if let Some(req) = requests.next() {
                    match self.ready.is_ready(req) {
                        Readiness::Ready => {
                            return Some(req.clone());
                        }
                        state => {
                            trace!(target: "pool", "[{:?}] Ignoring {:?} request.", req.hash(), state)
                        }
                    }
                }
            }

            let next_sender = self.senders.next()?;
            self.queues = Some(next_sender.1.iter())
        }
    }
}

/// An iterator over all pending (ready) requests.
/// NOTE: the requests are not removed from the queue.
/// You might remove them later by calling `cull`.
pub struct PendingIterator<'a, R, Ready, S>
where
    R: PoolRequest + 'a,
    Ready: ReadyChecker<R>,
    S: RequestSelector<R> + 'a,
{
    pub ready: Ready,
    pub best_requests: BTreeSet<ScoredRequest<S::Score, R>>,
    pub pool: &'a InnerPool<R, S>,
}

impl<'a, R, Ready, S> Iterator for PendingIterator<'a, R, Ready, S>
where
    R: PoolRequest + 'a,
    Ready: ReadyChecker<R>,
    S: RequestSelector<R>,
{
    type Item = R;

    fn next(&mut self) -> Option<Self::Item> {
        while !self.best_requests.is_empty() {
            let best = {
                let best = self
                    .best_requests
                    .iter()
                    .next()
                    .expect("best_requests is not empty; qed")
                    .clone();

                self.best_requests
                    .take(&best)
                    .expect("Just taken from iterator; qed")
            };

            match self.ready.is_ready(&best.request()) {
                Readiness::Ready => {
                    // retrieve next one from that sender.
                    let next = self
                        .pool
                        .queues
                        .get(best.request().sender())
                        .and_then(|s| s.find_next(&best.request(), &self.pool.selector));
                    if let Some((score, tx)) = next {
                        self.best_requests.insert(ScoredRequest::new(score, tx));
                    }

                    return Some(best.request().clone());
                }
                state => trace!(target: "pool",
                    "[{:?}] Ignoring {:?} transaction.",
                    best.request().hash(),
                    state
                ),
            }
        }

        None
    }
}
