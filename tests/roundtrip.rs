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

use std::fs;
use std::path::Path;

fn vector(name: &str) -> Vec<u8> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/vectors");
    fs::read(dir.join(format!("{name}.raw"))).unwrap()
}

fn round_trip(data: &[u8]) {
    let packed = lzhuf::compress(data);
    let restored = lzhuf::decompress(&packed, data.len());
    assert_eq!(restored, data);
}

#[test]
fn round_trip_empty() {
    round_trip(b"");
}

#[test]
fn round_trip_small_literal_run() {
    round_trip(b"a");
    round_trip(b"abcdefghij");
}

#[test]
fn round_trip_repeats_exercise_matches() {
    round_trip(&b"FCKAFD".repeat(64));
    round_trip(&[0u8; 5000]);
}

#[test]
fn round_trip_vectors() {
    for name in [
        "short",
        "repetitive",
        "random_small",
        "reconst_big",
        "mixed",
    ] {
        round_trip(&vector(name));
    }
}

#[test]
fn round_trip_all_byte_values() {
    let data: Vec<u8> = (0..=255u8).cycle().take(8192).collect();
    round_trip(&data);
}

#[test]
fn compress_then_decompress_is_shorter_for_redundant_data() {
    let data = b"Science class should not end in tragedy. ".repeat(256);
    let packed = lzhuf::compress(&data);
    assert!(packed.len() < data.len());
    assert_eq!(lzhuf::decompress(&packed, data.len()), data);
}
