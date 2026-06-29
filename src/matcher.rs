// This file is part of lzhuf.
//
// Copyright (c) 2026  René Coignard <contact@renecoignard.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::{LOOKAHEAD, MIN_MATCH, WINDOW_MASK, WINDOW_SIZE};

const HASH_BITS: u32 = 13;
const HASH_SIZE: usize = 1 << HASH_BITS;
const HASH_MULTIPLIER: u32 = 0x9E37_79B1;
const MAX_CHAIN: usize = 256;

pub(crate) struct Matcher {
    head: Vec<Option<usize>>,
    prev: Vec<Option<usize>>,
}

impl Matcher {
    pub(crate) fn new() -> Self {
        Self {
            head: vec![None; HASH_SIZE],
            prev: vec![None; WINDOW_SIZE],
        }
    }

    fn hash(window: &[u8]) -> usize {
        let key = ((window[0] as u32) << 16) | ((window[1] as u32) << 8) | (window[2] as u32);
        (key.wrapping_mul(HASH_MULTIPLIER) >> (32 - HASH_BITS)) as usize & (HASH_SIZE - 1)
    }

    pub(crate) fn insert(&mut self, input: &[u8], pos: usize) {
        if pos + MIN_MATCH <= input.len() {
            let slot = Self::hash(&input[pos..]);
            self.prev[pos & WINDOW_MASK] = self.head[slot];
            self.head[slot] = Some(pos);
        }
    }

    pub(crate) fn longest_match(&self, input: &[u8], pos: usize) -> Option<(usize, usize)> {
        if pos + MIN_MATCH > input.len() {
            return None;
        }
        let max_length = (input.len() - pos).min(LOOKAHEAD);
        let window_floor = pos.saturating_sub(WINDOW_SIZE - 1);
        let mut candidate = self.head[Self::hash(&input[pos..])];
        let mut probes = 0;
        let mut best_length = 0;
        let mut best_source = 0;

        while let Some(source) = candidate
            && source >= window_floor
            && probes < MAX_CHAIN
        {
            let mut length = 0;
            while length < max_length && input[source + length] == input[pos + length] {
                length += 1;
            }
            if length > best_length {
                best_length = length;
                best_source = source;
                if length == max_length {
                    break;
                }
            }
            candidate = self.prev[source & WINDOW_MASK];
            probes += 1;
        }

        (best_length >= MIN_MATCH).then(|| (best_length, pos - best_source - 1))
    }
}
