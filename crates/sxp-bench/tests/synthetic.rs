use std::fs;
use std::path::Path;

use sxp_bench::{polynomial_signal, recurrence_signal, sha256_hex, trigonometric_signal};

const POLYNOMIAL_SHA256: &str = "7364c598b868aae6247ed6554b1fc9e2e251220f237c913f0ac2720a5b2b5e92";
const TRIGONOMETRIC_SHA256: &str =
    "1a2fa720c8ec350f1519e98632eae12add852e2d73c4ce931fd1d91129e93fcb";
const RECURRENCE_SHA256: &str = "90b1408a5ea7b47f68e2ed160763d5f1d196a4533e74ad67a357005466a1c291";

fn manifest_rows() -> Vec<(String, String)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpora/manifest.toml");
    let text = fs::read_to_string(path).unwrap();
    let table: toml::Table = text.parse().unwrap();
    let inputs = table.get("input").unwrap().as_array().unwrap();
    inputs
        .iter()
        .map(|value| {
            let row = value.as_table().unwrap();
            let id = row.get("id").unwrap().as_str().unwrap().to_string();
            let hash = row
                .get("derived_sha256")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();
            (id, hash)
        })
        .collect()
}

fn assert_signal(id: &str, bytes: &[u8], literal: &str) {
    assert_eq!(bytes.len(), 4096);
    assert_eq!(&bytes[..2048], &bytes[2048..]);
    let hash = sha256_hex(bytes).unwrap();
    assert_eq!(hash, literal);
    let rows = manifest_rows();
    let recorded = rows
        .iter()
        .find(|(row_id, _)| row_id == id)
        .map(|(_, row_hash)| row_hash.as_str())
        .unwrap();
    assert_eq!(recorded, literal);
}

fn assert_integers(parameters: &toml::Table, expected: &[(&str, i64)]) {
    for (key, value) in expected {
        assert_eq!(
            parameters.get(*key).and_then(|item| item.as_integer()),
            Some(*value)
        );
    }
}

fn assert_array(parameters: &toml::Table, key: &str, expected: &[i64]) {
    let values = parameters
        .get(key)
        .and_then(|item| item.as_array())
        .unwrap();
    let got: Vec<i64> = values
        .iter()
        .map(|item| item.as_integer().unwrap())
        .collect();
    assert_eq!(got, expected);
}

fn assert_parameters(id: &str, expected: &[(&str, i64)], arrays: &[(&str, &[i64])]) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpora/manifest.toml");
    let text = fs::read_to_string(path).unwrap();
    let table: toml::Table = text.parse().unwrap();
    let inputs = table.get("input").unwrap().as_array().unwrap();
    let row = inputs
        .iter()
        .map(|value| value.as_table().unwrap())
        .find(|row| row.get("id").and_then(|value| value.as_str()) == Some(id))
        .unwrap();
    let parameters = row.get("parameters").unwrap().as_table().unwrap();
    assert_integers(parameters, expected);
    for (key, values) in arrays {
        assert_array(parameters, key, values);
    }
}

#[test]
fn polynomial_matches_manifest() {
    assert_signal(
        "synthetic-polynomial",
        &polynomial_signal().unwrap(),
        POLYNOMIAL_SHA256,
    );
    assert_parameters(
        "synthetic-polynomial",
        &[("degree", 1), ("q", 8), ("blocks", 2)],
        &[("coefficients", &[0, 65_536])],
    );
}

#[test]
fn trigonometric_matches_manifest() {
    assert_signal(
        "synthetic-trigonometric",
        &trigonometric_signal().unwrap(),
        TRIGONOMETRIC_SHA256,
    );
    assert_parameters(
        "synthetic-trigonometric",
        &[
            ("order", 1),
            ("q", 8),
            ("omega", 0x0100_0000),
            ("blocks", 2),
        ],
        &[("coefficients", &[32_768, 32_000, 0])],
    );
}

#[test]
fn recurrence_matches_manifest() {
    assert_signal(
        "synthetic-recurrence",
        &recurrence_signal().unwrap(),
        RECURRENCE_SHA256,
    );
    assert_parameters(
        "synthetic-recurrence",
        &[("order", 2), ("m_mod", 256), ("blocks", 2)],
        &[("coefficients", &[1, 1, 0]), ("state", &[0, 1])],
    );
}
