//! Shared vocabulary for every hc crate: the typed intermediate representation,
//! RFC 8785 canonical encoding, SHA-256 digests and a hermetic git wrapper.

pub mod canon;
pub mod digest;
pub mod git;
pub mod ir;

pub use digest::Digest;
