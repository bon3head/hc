//! Canonical bytes per RFC 8785 (JSON Canonicalization Scheme) via `serde_jcs`,
//! checked against the cyberphone reference vectors in `refs/specs/jcs/testdata`.

use serde::Serialize;

use crate::Digest;

pub use serde_jcs::to_vec;

/// Largest integer magnitude I-JSON (RFC 7493 §2.2) and therefore RFC 8785 can carry exactly.
pub const MAX_SAFE_INTEGER: i64 = (1 << 53) - 1;

pub fn digest<T: Serialize + ?Sized>(value: &T) -> Result<Digest, serde_json::Error> {
    Ok(Digest::of(&to_vec(value)?))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::Path;

    use proptest::prelude::*;

    use super::to_vec;

    fn vectors() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../refs/specs/jcs/testdata")
    }

    #[test]
    fn rfc8785_reference_vectors() {
        let dir = vectors();
        let mut checked = 0;
        for entry in std::fs::read_dir(dir.join("input")).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap();
            let input: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            let expected = std::fs::read(dir.join("output").join(name)).unwrap();
            assert_eq!(to_vec(&input).unwrap(), expected, "vector {}", name.to_string_lossy());
            checked += 1;
        }
        assert_eq!(checked, 6);
    }

    #[test]
    fn integers_without_an_exact_double_are_rejected() {
        let two_53 = super::MAX_SAFE_INTEGER + 1;
        assert!(to_vec(&serde_json::json!({"n": two_53})).is_ok(), "2^53 is exactly representable");
        assert!(to_vec(&serde_json::json!({"n": two_53 + 1})).is_err(), "2^53+1 is not");
    }

    proptest! {
        #[test]
        fn insertion_order_does_not_matter(pairs in proptest::collection::vec(("[a-z]{1,6}", any::<i32>()), 0..12)) {
            let forward: BTreeMap<_, _> = pairs.iter().cloned().collect();
            let mut reversed = serde_json::Map::new();
            for (k, v) in forward.iter().rev() {
                reversed.insert(k.clone(), (*v).into());
            }
            prop_assert_eq!(to_vec(&forward).unwrap(), to_vec(&reversed).unwrap());
        }

        #[test]
        fn canonical_form_is_a_fixed_point(s in "\\PC{0,40}", n in -super::MAX_SAFE_INTEGER..=super::MAX_SAFE_INTEGER) {
            let value = serde_json::json!({"s": s, "n": n, "a": [n, {"z": s}]});
            let once = to_vec(&value).unwrap();
            let reparsed: serde_json::Value = serde_json::from_slice(&once).unwrap();
            prop_assert_eq!(to_vec(&reparsed).unwrap(), once);
        }
    }
}
