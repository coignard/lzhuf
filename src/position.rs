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

const POSITION_HIGH_BITS: usize = 6;
const POSITION_LOW_BITS: usize = 6;
const LEADING_BITS: usize = 8;
const POSITION_VALUES: usize = 64;
const POSITION_CODE_RUNS: [(u8, usize); 6] = [(3, 1), (4, 3), (5, 8), (6, 12), (7, 24), (8, 16)];

pub(crate) struct PositionTable {
    value: [u8; 256],
    length: [u8; 256],
    code: [u8; POSITION_VALUES],
    code_length: [u8; POSITION_VALUES],
}

impl PositionTable {
    pub(crate) fn build() -> Self {
        let mut value = [0u8; 256];
        let mut length = [0u8; 256];
        let mut code = [0u8; POSITION_VALUES];
        let mut code_length = [0u8; POSITION_VALUES];

        let mut index = 0usize;
        let mut symbol = 0usize;
        let mut prefix = 0u16;
        for (bit_length, symbol_count) in POSITION_CODE_RUNS {
            for _ in 0..symbol_count {
                code[symbol] = prefix as u8;
                code_length[symbol] = bit_length;
                prefix += 1 << (LEADING_BITS - bit_length as usize);

                for _ in 0..(1usize << (LEADING_BITS - bit_length as usize)) {
                    value[index] = symbol as u8;
                    length[index] = bit_length;
                    index += 1;
                }
                symbol += 1;
            }
        }
        Self {
            value,
            length,
            code,
            code_length,
        }
    }

    pub(crate) fn decode(&self, reader: &mut BitReader<'_>) -> Option<usize> {
        let leading = reader.byte()?;
        let high = (self.value[leading] as usize) << POSITION_LOW_BITS;
        let extra = self.length[leading] as usize - (LEADING_BITS - POSITION_HIGH_BITS);
        let mut low = leading;
        for _ in 0..extra {
            low = (low << 1) + reader.bit()? as usize;
        }
        Some(high | (low & ((1 << POSITION_LOW_BITS) - 1)))
    }

    pub(crate) fn encode(&self, writer: &mut BitWriter, distance: usize) {
        let high = distance >> POSITION_LOW_BITS;
        let length = self.code_length[high] as u32;
        writer.put(
            (self.code[high] >> (LEADING_BITS as u32 - length)) as u32,
            length,
        );
        writer.put(
            (distance & ((1 << POSITION_LOW_BITS) - 1)) as u32,
            POSITION_LOW_BITS as u32,
        );
    }
}
