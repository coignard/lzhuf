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

fn check(name: &str) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/vectors");
    let raw = fs::read(dir.join(format!("{name}.raw"))).unwrap();
    let compressed = fs::read(dir.join(format!("{name}.lzh"))).unwrap();
    let decoded = lzhuf::decompress(&compressed, raw.len() + 16);
    assert_eq!(decoded, raw, "{name}");
}

#[test]
fn short_literal_and_match_mix() {
    check("short");
}

#[test]
fn highly_repetitive_payload() {
    check("repetitive");
}

#[test]
fn incompressible_random() {
    check("random_small");
}

#[test]
fn large_input_triggers_tree_reconstruction() {
    check("reconst_big");
}

#[test]
fn mixed_redundancy() {
    check("mixed");
}

#[test]
fn empty_input_yields_empty_output() {
    assert!(lzhuf::decompress(&[], 1024).is_empty());
}

#[test]
fn output_is_bounded_by_max_output() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/vectors");
    let compressed = fs::read(dir.join("reconst_big.lzh")).unwrap();
    assert_eq!(lzhuf::decompress(&compressed, 100).len(), 100);
}

#[test]
fn truncated_stream_does_not_panic() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/vectors");
    let compressed = fs::read(dir.join("mixed.lzh")).unwrap();
    for cut in 0..compressed.len().min(64) {
        let _ = lzhuf::decompress(&compressed[..cut], 64 * 1024);
    }
}
