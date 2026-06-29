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

pub(crate) struct BitReader<'a> {
    bytes: &'a [u8],
    byte_pos: usize,
    bit_pos: u8,
}

impl<'a> BitReader<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            byte_pos: 0,
            bit_pos: 0,
        }
    }

    pub(crate) fn bit(&mut self) -> Option<u8> {
        let byte = *self.bytes.get(self.byte_pos)?;
        let value = (byte >> (7 - self.bit_pos)) & 1;
        self.bit_pos += 1;
        if self.bit_pos == 8 {
            self.bit_pos = 0;
            self.byte_pos += 1;
        }
        Some(value)
    }

    pub(crate) fn byte(&mut self) -> Option<usize> {
        let mut value = 0usize;
        for _ in 0..8 {
            value = (value << 1) | self.bit()? as usize;
        }
        Some(value)
    }
}

pub(crate) struct BitWriter {
    bytes: Vec<u8>,
    accumulator: u8,
    filled: u8,
}

impl BitWriter {
    pub(crate) fn new() -> Self {
        Self {
            bytes: Vec::new(),
            accumulator: 0,
            filled: 0,
        }
    }

    pub(crate) fn put(&mut self, value: u32, count: u32) {
        for shift in (0..count).rev() {
            self.accumulator = (self.accumulator << 1) | ((value >> shift) & 1) as u8;
            self.filled += 1;
            if self.filled == 8 {
                self.bytes.push(self.accumulator);
                self.accumulator = 0;
                self.filled = 0;
            }
        }
    }

    pub(crate) fn finish(mut self) -> Vec<u8> {
        if self.filled > 0 {
            self.accumulator <<= 8 - self.filled;
            self.bytes.push(self.accumulator);
        }
        self.bytes
    }
}
