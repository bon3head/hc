use bedrock::ir::{ClaimKind, Trust};
use crossexam::Verified;

fn main() {
    let _forged = Verified {
        claim_seq: 0,
        claim_id: "c1".into(),
        kind: ClaimKind::TestsPass,
        trust: Trust::Reproduced,
        evidence: vec![],
    };
}
