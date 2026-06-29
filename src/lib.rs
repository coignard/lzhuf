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

mod bits;
mod huffman;
mod matcher;
mod position;

use bits::{BitReader, BitWriter};
use huffman::AdaptiveHuffman;
use matcher::Matcher;
use position::PositionTable;

pub(crate) const WINDOW_SIZE: usize = 4096;
pub(crate) const WINDOW_MASK: usize = WINDOW_SIZE - 1;
pub(crate) const LOOKAHEAD: usize = 60;
pub(crate) const THRESHOLD: usize = 2;
pub(crate) const MIN_MATCH: usize = THRESHOLD + 1;
pub(crate) const SYMBOL_COUNT: usize = 256 - THRESHOLD + LOOKAHEAD;
pub(crate) const TABLE_SIZE: usize = SYMBOL_COUNT * 2 - 1;
pub(crate) const ROOT: usize = TABLE_SIZE - 1;
pub(crate) const MAX_FREQ: u16 = 0x8000;

const LITERAL_LIMIT: usize = 256;
const MATCH_SYMBOL_BASE: usize = 255 - THRESHOLD;
const RING_FILL: u8 = b' ';

pub fn compress(input: &[u8]) -> Vec<u8> {
    let position = PositionTable::build();
    let mut tree = AdaptiveHuffman::new();
    let mut writer = BitWriter::new();
    let mut matcher = Matcher::new();

    let mut pos = 0;
    while pos < input.len() {
        if let Some((length, distance)) = matcher.longest_match(input, pos) {
            tree.encode_symbol(&mut writer, length + MATCH_SYMBOL_BASE);
            position.encode(&mut writer, distance);
            for offset in 0..length {
                matcher.insert(input, pos + offset);
            }
            pos += length;
        } else {
            tree.encode_symbol(&mut writer, input[pos] as usize);
            matcher.insert(input, pos);
            pos += 1;
        }
    }
    writer.finish()
}

pub fn decompress(input: &[u8], max_output: usize) -> Vec<u8> {
    let position = PositionTable::build();
    let mut tree = AdaptiveHuffman::new();
    let mut reader = BitReader::new(input);
    let mut ring = [RING_FILL; WINDOW_SIZE];
    let mut cursor = WINDOW_SIZE - LOOKAHEAD;
    let mut output = Vec::new();

    while output.len() < max_output {
        let Some(symbol) = tree.decode_symbol(&mut reader) else {
            break;
        };
        if symbol < LITERAL_LIMIT {
            let byte = symbol as u8;
            output.push(byte);
            ring[cursor] = byte;
            cursor = (cursor + 1) & WINDOW_MASK;
        } else {
            let Some(distance) = position.decode(&mut reader) else {
                break;
            };
            let mut source = (cursor + WINDOW_SIZE - distance - 1) & WINDOW_MASK;
            let length = symbol - MATCH_SYMBOL_BASE;
            for _ in 0..length {
                let byte = ring[source];
                output.push(byte);
                ring[cursor] = byte;
                cursor = (cursor + 1) & WINDOW_MASK;
                source = (source + 1) & WINDOW_MASK;
                if output.len() >= max_output {
                    break;
                }
            }
        }
    }
    output
}
