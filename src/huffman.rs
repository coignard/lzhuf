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

use crate::bits::{BitReader, BitWriter};
use crate::{MAX_FREQ, ROOT, SYMBOL_COUNT, TABLE_SIZE};

const FREQ_SENTINEL: u16 = 0xFFFF;
const CODE_BITS: u32 = 32;
const CODE_TOP_BIT: u32 = 1 << (CODE_BITS - 1);

pub(crate) struct AdaptiveHuffman {
    freq: [u16; TABLE_SIZE + 1],
    parent: [u16; TABLE_SIZE + SYMBOL_COUNT],
    son: [u16; TABLE_SIZE],
}

impl AdaptiveHuffman {
    pub(crate) fn new() -> Self {
        let mut tree = Self {
            freq: [0; TABLE_SIZE + 1],
            parent: [0; TABLE_SIZE + SYMBOL_COUNT],
            son: [0; TABLE_SIZE],
        };
        for symbol in 0..SYMBOL_COUNT {
            tree.freq[symbol] = 1;
            tree.son[symbol] = (symbol + TABLE_SIZE) as u16;
            tree.parent[symbol + TABLE_SIZE] = symbol as u16;
        }
        let mut leaf = 0usize;
        let mut node = SYMBOL_COUNT;
        while node <= ROOT {
            tree.freq[node] = tree.freq[leaf] + tree.freq[leaf + 1];
            tree.son[node] = leaf as u16;
            tree.parent[leaf] = node as u16;
            tree.parent[leaf + 1] = node as u16;
            leaf += 2;
            node += 1;
        }
        tree.freq[TABLE_SIZE] = FREQ_SENTINEL;
        tree.parent[ROOT] = 0;
        tree
    }

    fn reconstruct(&mut self) {
        let mut packed = 0usize;
        for node in 0..TABLE_SIZE {
            if self.son[node] as usize >= TABLE_SIZE {
                self.freq[packed] = self.freq[node].div_ceil(2);
                self.son[packed] = self.son[node];
                packed += 1;
            }
        }
        let mut child = 0usize;
        let mut node = SYMBOL_COUNT;
        while node < TABLE_SIZE {
            let combined = self.freq[child] + self.freq[child + 1];
            let mut insert = 0usize;
            let mut scan = node - 1;
            loop {
                if self.freq[scan] <= combined {
                    insert = scan + 1;
                    break;
                }
                if scan == 0 {
                    break;
                }
                scan -= 1;
            }
            for slot in (insert..node).rev() {
                self.freq[slot + 1] = self.freq[slot];
                self.son[slot + 1] = self.son[slot];
            }
            self.freq[insert] = combined;
            self.son[insert] = child as u16;
            child += 2;
            node += 1;
        }
        for node in 0..TABLE_SIZE {
            let son = self.son[node] as usize;
            self.parent[son] = node as u16;
            if son < TABLE_SIZE {
                self.parent[son + 1] = node as u16;
            }
        }
    }

    fn update(&mut self, symbol: usize) {
        if self.freq[ROOT] == MAX_FREQ {
            self.reconstruct();
        }
        let mut node = self.parent[symbol + TABLE_SIZE] as usize;
        loop {
            self.freq[node] += 1;
            let raised = self.freq[node];
            let mut next = node + 1;
            if raised > self.freq[next] {
                next += 1;
                while raised > self.freq[next] {
                    next += 1;
                }
                next -= 1;
                self.freq[node] = self.freq[next];
                self.freq[next] = raised;

                let left = self.son[node] as usize;
                self.parent[left] = next as u16;
                if left < TABLE_SIZE {
                    self.parent[left + 1] = next as u16;
                }
                let swapped = self.son[next] as usize;
                self.son[next] = left as u16;
                self.parent[swapped] = node as u16;
                if swapped < TABLE_SIZE {
                    self.parent[swapped + 1] = node as u16;
                }
                self.son[node] = swapped as u16;

                node = next;
            }
            node = self.parent[node] as usize;
            if node == 0 {
                break;
            }
        }
    }

    pub(crate) fn decode_symbol(&mut self, reader: &mut BitReader<'_>) -> Option<usize> {
        let mut node = self.son[ROOT] as usize;
        while node < TABLE_SIZE {
            node += reader.bit()? as usize;
            node = self.son[node] as usize;
        }
        let symbol = node - TABLE_SIZE;
        self.update(symbol);
        Some(symbol)
    }

    pub(crate) fn encode_symbol(&mut self, writer: &mut BitWriter, symbol: usize) {
        let mut code = 0u32;
        let mut length = 0u32;
        let mut node = self.parent[symbol + TABLE_SIZE] as usize;
        loop {
            code >>= 1;
            if node & 1 != 0 {
                code |= CODE_TOP_BIT;
            }
            length += 1;
            node = self.parent[node] as usize;
            if node == ROOT {
                break;
            }
        }
        writer.put(code >> (CODE_BITS - length), length);
        self.update(symbol);
    }
}
