use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest as _, Sha256};

/// A SHA-256 digest, written `sha256:<lowercase hex>`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Digest([u8; 32]);

impl Digest {
    pub fn of(bytes: &[u8]) -> Self {
        Self(Sha256::digest(bytes).into())
    }

    pub fn hex(&self) -> String {
        hex::encode(self.0)
    }

    pub const fn zero() -> Self {
        Self([0; 32])
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sha256:{}", self.hex())
    }
}

impl fmt::Debug for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid digest {0:?}: expected sha256:<64 hex chars>")]
pub struct DigestParseError(String);

impl FromStr for Digest {
    type Err = DigestParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex_part = s.strip_prefix("sha256:").ok_or_else(|| DigestParseError(s.into()))?;
        let mut out = [0u8; 32];
        hex::decode_to_slice(hex_part, &mut out).map_err(|_| DigestParseError(s.into()))?;
        Ok(Self(out))
    }
}

impl Serialize for Digest {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Digest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::Digest;

    #[test]
    fn known_vector() {
        assert_eq!(Digest::of(b"abc").to_string(), "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }

    #[test]
    fn round_trips() {
        let d = Digest::of(b"x");
        assert_eq!(d.to_string().parse::<Digest>().unwrap(), d);
        assert!("sha256:zz".parse::<Digest>().is_err());
    }
}
