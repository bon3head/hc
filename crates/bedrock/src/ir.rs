//! The typed intermediate representation. Every ledger entry is an in-toto
//! Statement v1 (`refs/specs/in-toto/statement.md`) whose predicate is one of
//! three kinds, each with exactly one permitted signer role.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::Digest;

pub const STATEMENT_TYPE: &str = "https://in-toto.io/Statement/v1";
pub const PREDICATE_OBS: &str = "urn:hc:predicate:obs:v0";
pub const PREDICATE_CLAIM: &str = "urn:hc:predicate:claim:v0";
pub const PREDICATE_VERDICT: &str = "urn:hc:predicate:verdict:v0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceDescriptor {
    pub name: String,
    pub digest: BTreeMap<String, String>,
}

impl ResourceDescriptor {
    pub fn sha256(name: impl Into<String>, digest: Digest) -> Self {
        Self { name: name.into(), digest: BTreeMap::from([("sha256".to_owned(), digest.hex())]) }
    }

    /// `algo` is one of the in-toto git digest names: `gitCommit`, `gitTree`, `gitBlob`.
    pub fn git(name: impl Into<String>, algo: &str, oid: &str) -> Self {
        Self { name: name.into(), digest: BTreeMap::from([(algo.to_owned(), oid.to_owned())]) }
    }
}

/// A git object id (hex), as printed by git.
pub type Oid = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Statement<P> {
    #[serde(rename = "_type")]
    pub type_: String,
    pub subject: Vec<ResourceDescriptor>,
    #[serde(rename = "predicateType")]
    pub predicate_type: String,
    pub predicate: P,
}

impl<P> Statement<P> {
    pub fn new(predicate_type: &str, subject: Vec<ResourceDescriptor>, predicate: P) -> Self {
        Self { type_: STATEMENT_TYPE.to_owned(), subject, predicate_type: predicate_type.to_owned(), predicate }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Agent,
    Harness,
    Kernel,
}

/// How an agent identity was established, strongest first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttestationLevel {
    /// `hc run` launched the agent process and captured its output itself.
    Launcher,
    /// `hc` was invoked from inside a process tree containing a registered agent executable.
    Ancestry,
    /// No registered agent was found; the caller named itself.
    Declared,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentAttestation {
    pub agent_id: String,
    pub level: AttestationLevel,
    pub executable: Option<String>,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Isolation {
    Bwrap,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Obs {
    pub session: String,
    pub ts: String,
    #[serde(flatten)]
    pub event: Event,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Event {
    SessionStart {
        task: String,
        agent: AgentAttestation,
        isolation: Isolation,
        tree: Oid,
        head: Option<Oid>,
    },
    FileWrite {
        path: String,
        before: Option<Oid>,
        after: Option<Oid>,
        added_lines: Vec<u32>,
    },
    SessionEnd {
        exit_code: Option<i32>,
        tree: Oid,
    },
    NoClaim {
        diagnostics: Vec<Diagnostic>,
    },
    /// The agent's own plan as the harness read it from a native plan tool.
    PlanUpdate {
        source: String,
        steps: Vec<PlanStep>,
    },
    /// A mid-session snapshot taken at a step boundary.
    Checkpoint {
        tree: Oid,
    },
    /// The starting point the owner accepted with `hc init`: the code as it was, and the
    /// task contract that governs everything recorded after it. Writes are attributed to
    /// agents from here on, whichever session records them.
    Baseline {
        tree: Oid,
        head: Option<Oid>,
        task: serde_json::Value,
        recorded_by: AgentAttestation,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
    Blocked,
}

/// One plan step. `key` is stable across plan rewrites (native id when the
/// harness provides one, otherwise derived by `agenda`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanStep {
    pub key: String,
    pub text: String,
    pub status: StepStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimKind {
    FilesChanged,
    FilesUntouched,
    CommandRan,
    TestsPass,
    RequirementMet,
    DerivedNotRetrieved,
    NoChangeNeeded,
    SpecConflict,
    Blocked,
    Note,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Done,
    NoChangeNeeded,
    SpecConflict,
    Blocked,
}

/// One claim exactly as the agent-facing schema (`schema/claim.v0.json`) defines it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: ClaimKind,
    pub basis: String,
    pub command: Option<String>,
    pub exit_code: Option<i64>,
    pub passed: Option<u64>,
    pub failed: Option<u64>,
    pub paths: Vec<String>,
    pub commit: Option<String>,
    pub depends_on: Vec<String>,
    pub status: Status,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimEnvelope {
    pub task: String,
    pub claims: Vec<Claim>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimRecord {
    pub session: String,
    pub task: String,
    pub attestation: AgentAttestation,
    pub claim: Claim,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Trust {
    Refuted,
    Asserted,
    Observed,
    Reproduced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PassId {
    #[serde(rename = "command")]
    Command,
    #[serde(rename = "tamper")]
    Tamper,
    #[serde(rename = "scope")]
    Scope,
    #[serde(rename = "signature")]
    Signature,
    #[serde(rename = "reexec")]
    Reexec,
    #[serde(rename = "falsify")]
    Falsify,
}

impl PassId {
    pub const ALL: [Self; 6] = [Self::Command, Self::Tamper, Self::Scope, Self::Signature, Self::Reexec, Self::Falsify];

    /// The pass number in the research design (P4 command binding .. P8 re-execution).
    pub const fn number(self) -> &'static str {
        match self {
            Self::Command => "P4",
            Self::Tamper => "P6",
            Self::Scope => "P6S",
            Self::Signature => "P7",
            Self::Reexec => "P8",
            Self::Falsify => "P9",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Pass,
    Fail,
    /// The check could not obtain evidence (for example, the test runner did not start).
    /// It neither supports nor contradicts the claim.
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Check {
    pub pass: PassId,
    pub result: Outcome,
    pub rule: Option<String>,
    pub evidence: Vec<u64>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerdictRecord {
    pub session: String,
    pub about: String,
    pub claim_seq: u64,
    pub checks: Vec<Check>,
    pub trust: Trust,
    pub policy: Digest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub path: String,
}

impl Diagnostic {
    pub fn new(code: &str, message: impl Into<String>, path: impl Into<String>) -> Self {
        Self { code: code.to_owned(), message: message.into(), path: path.into() }
    }
}

/// A decoded predicate together with the role that signed it. Serialized only for
/// display (`hc show`); the ledger stores each predicate as its own statement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Body {
    Obs(Obs),
    Claim(ClaimRecord),
    Verdict(VerdictRecord),
}

impl Body {
    pub const fn predicate_type(&self) -> &'static str {
        match self {
            Self::Obs(_) => PREDICATE_OBS,
            Self::Claim(_) => PREDICATE_CLAIM,
            Self::Verdict(_) => PREDICATE_VERDICT,
        }
    }

    pub const fn required_role(&self) -> Role {
        match self {
            Self::Obs(_) => Role::Harness,
            Self::Claim(_) => Role::Agent,
            Self::Verdict(_) => Role::Kernel,
        }
    }

    pub fn session(&self) -> &str {
        match self {
            Self::Obs(o) => &o.session,
            Self::Claim(c) => &c.session,
            Self::Verdict(v) => &v.session,
        }
    }
}
