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
pub struct Params {
    pub max_count: usize,
    pub max_per_sender: usize,
    pub max_mem_usage: usize,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            max_count: 10240,
            max_per_sender: 16,
            max_mem_usage: 8 * 1024 * 1024,
        }
    }
}
