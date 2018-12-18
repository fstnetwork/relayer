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
use smallvec::SmallVec;
use std::collections::HashSet;
use std::mem;

use super::{Change, Choice, PoolRequest, Readiness, ReadyChecker, RequestSelector};

pub enum AddResult<R, S> {
    Ok(R),
    TooCheapToEnter(R, S),
    TooCheap { old: R, new: R },
    Replaced { old: R, new: R },
    PushedOut { old: R, new: R },
}

const PER_SENDER: usize = 8;

pub struct RequestQueue<R, S>
where
    R: PoolRequest,
    S: RequestSelector<R>,
{
    requests: SmallVec<[R; PER_SENDER]>,
    scores: SmallVec<[S::Score; PER_SENDER]>,
}

impl<R, S> Default for RequestQueue<R, S>
where
    R: PoolRequest,
    S: RequestSelector<R>,
{
    fn default() -> RequestQueue<R, S> {
        RequestQueue {
            requests: Default::default(),
            scores: Default::default(),
        }
    }
}

impl<R, S> RequestQueue<R, S>
where
    R: PoolRequest,
    S: RequestSelector<R>,
{
    pub fn new() -> RequestQueue<R, S> {
        RequestQueue::default()
    }

    pub fn find_next(&self, request: &R, selector: &S) -> Option<(S::Score, R)> {
        self.requests
            .binary_search_by(|old| selector.compare(old, &request))
            .ok()
            .and_then(|index| {
                let index = index + 1;
                if index < self.scores.len() {
                    Some((self.scores[index].clone(), self.requests[index].clone()))
                } else {
                    None
                }
            })
    }

    pub fn worst_and_best(&self) -> Option<((S::Score, R), (S::Score, R))> {
        let len = self.requests.len();
        self.scores.get(0).cloned().map(|best_score| {
            let worst_score = self.scores[len - 1].clone();
            let worst_req = self.requests[len - 1].clone();
            let best_req = self.requests[0].clone();
            ((worst_score, worst_req), (best_score, best_req))
        })
    }

    fn push_cheapest(
        &mut self,
        request: R,
        selector: &S,
        max_count: usize,
    ) -> AddResult<R, S::Score> {
        let index = self.requests.len();
        if index == max_count && !selector.should_ignore_sender_limit(&request) {
            let min_score = self.scores[index - 1].clone();
            AddResult::TooCheapToEnter(request, min_score)
        } else {
            self.requests.push(request.clone());
            self.scores.push(Default::default());

            selector.update_scores(&self.requests, &mut self.scores, Change::InsertedAt(index));
            AddResult::Ok(request)
        }
    }

    pub fn add(&mut self, request: R, selector: &S, max_count: usize) -> AddResult<R, S::Score> {
        let index = match self
            .requests
            .binary_search_by(|old| selector.compare(old, &request))
        {
            Ok(index) => index,
            Err(index) => index,
        };

        if index == self.requests.len() {
            return self.push_cheapest(request, selector, max_count);
        }

        match selector.choose(&self.requests[index], &request) {
            Choice::InsertNew => {
                self.requests.insert(index, request.clone());
                self.scores.insert(index, Default::default());
                selector.update_scores(&self.requests, &mut self.scores, Change::InsertedAt(index));

                if self.requests.len() > max_count {
                    let old = self.requests.pop().expect("len is non-zero; qed");
                    self.scores.pop();
                    selector.update_scores(
                        &self.requests,
                        &mut self.scores,
                        Change::RemovedAt(self.requests.len()),
                    );

                    AddResult::PushedOut { old, new: request }
                } else {
                    AddResult::Ok(request)
                }
            }
            Choice::RejectNew => AddResult::TooCheap {
                old: self.requests[index].clone(),
                new: request,
            },
            Choice::ReplaceOld => {
                let old = mem::replace(&mut self.requests[index], request.clone());
                selector.update_scores(&self.requests, &mut self.scores, Change::ReplacedAt(index));

                AddResult::Replaced { old, new: request }
            }
        }
    }

    pub fn remove(&mut self, request: &R, selector: &S) -> bool {
        let index = match self
            .requests
            .binary_search_by(|old| selector.compare(old, &request))
        {
            Ok(index) => index,
            Err(_) => {
                warn!("Attempting to remove non-existent request {:?}", request);
                return false;
            }
        };

        self.requests.remove(index);
        self.scores.remove(index);
        selector.update_scores(&self.requests, &mut self.scores, Change::RemovedAt(index));

        true
    }

    pub fn cull<Ready: ReadyChecker<R>>(
        &mut self,
        ready: &mut Ready,
        selector: &S,
    ) -> SmallVec<[R; PER_SENDER]> {
        let mut result = SmallVec::default();
        if self.requests.is_empty() {
            return result;
        }

        let mut first_non_stalled = 0;
        for req in &self.requests {
            match ready.is_ready(req) {
                Readiness::Stale => {
                    first_non_stalled += 1;
                }
                Readiness::Ready | Readiness::Future => break,
            }
        }

        if first_non_stalled == 0 {
            return result;
        }

        // reverse the vectors to easily remove first elements.
        self.requests.reverse();
        self.scores.reverse();

        for _ in 0..first_non_stalled {
            self.scores.pop();
            result.push(
                self.requests
                    .pop()
                    .expect("first_non_stalled is never greater than requests.len(); qed"),
            );
        }

        self.requests.reverse();
        self.scores.reverse();

        // update scoring
        selector.update_scores(
            &self.requests,
            &mut self.scores,
            Change::Culled(result.len()),
        );

        // reverse the result to maintain correct order.
        result.reverse();
        result
    }

    #[allow(unused)]
    pub fn clear(&mut self) {
        self.requests.clear();
        self.scores.clear();
    }

    pub fn update_scores(&mut self, selector: &S, event: S::Event) {
        selector.update_scores(&self.requests, &mut self.scores, Change::Event(event));
    }

    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }

    pub fn len(&self) -> usize {
        self.requests.len()
    }

    pub fn iter(&self) -> ::std::slice::Iter<R> {
        self.requests.iter()
    }

    pub fn tokens(&self) -> HashSet<Address> {
        self.requests.iter().fold(HashSet::new(), |mut m, req| {
            m.insert(*req.token());
            m
        })
    }
}
