//! `Verified` is an abstract type in the LCF sense: its private seal field
//! means no code outside this crate can construct one, so a fact can enter a
//! brief only by passing through the kernel (`tests/ui/forge_verified.rs`
//! proves the compiler rejects the attempt).

use bedrock::ir::{ClaimKind, Trust};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Seal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verified {
    claim_seq: u64,
    claim_id: String,
    kind: ClaimKind,
    trust: Trust,
    evidence: Vec<u64>,
    seal: Seal,
}

impl Verified {
    pub(crate) fn mint(claim_seq: u64, claim_id: String, kind: ClaimKind, trust: Trust, evidence: Vec<u64>) -> Option<Self> {
        (trust >= Trust::Observed).then_some(Self { claim_seq, claim_id, kind, trust, evidence, seal: Seal })
    }

    pub fn claim_seq(&self) -> u64 {
        self.claim_seq
    }

    pub fn claim_id(&self) -> &str {
        &self.claim_id
    }

    pub fn kind(&self) -> ClaimKind {
        self.kind
    }

    pub fn trust(&self) -> Trust {
        self.trust
    }

    pub fn evidence(&self) -> &[u64] {
        &self.evidence
    }
}
