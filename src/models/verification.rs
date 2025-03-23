use std::{ fmt, time::Duration };
use crate::models::common::{ ProofLevel, ResourceLimits, ResourceUsage };

/// Result of the verification process
pub struct VerificationResult {
    pub status: VerificationStatus,
    pub proof_artifacts: Vec<ProofArtifact>,
    pub verification_time: Duration,
    pub resource_usage: ResourceUsage,
}

impl fmt::Debug for VerificationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VerificationStatus::Verified => write!(f, "Verified"),
            VerificationStatus::Unverified => write!(f, "Unverified"),
            VerificationStatus::Failed(reasons) => { write!(f, "Failed({:?})", reasons) }
            VerificationStatus::Timeout => write!(f, "Timeout"),
            VerificationStatus::Error(msg) => write!(f, "Error({:?})", msg),
        }
    }
}

impl fmt::Display for VerificationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VerificationStatus::Verified => write!(f, "Verified"),
            VerificationStatus::Unverified => write!(f, "Unverified"),
            VerificationStatus::Failed(reasons) => {
                if reasons.is_empty() {
                    write!(f, "Failed")
                } else {
                    write!(f, "Failed: {}", reasons.join(", "))
                }
            }
            VerificationStatus::Timeout => write!(f, "Timeout"),
            VerificationStatus::Error(msg) => write!(f, "Error: {}", msg),
        }
    }
}
/// Status of a verification attempt
pub enum VerificationStatus {
    Verified, // Successfully verified
    Unverified, // Verification incomplete
    Failed(Vec<String>), // Verification failed with reasons
    Timeout, // Verification timed out
    Error(String), // Error during verification
}

/// Artifacts produced during the verification process
#[derive(Debug)]
pub struct ProofArtifact {
    pub artifact_type: ArtifactType,
    pub path: String,
    pub description: String,
}

/// Types of proof artifacts
#[derive(Debug)]
pub enum ArtifactType {
    Proof,
    Model,
    Counterexample,
    Log,
    Custom(String),
}

/// Options for the verification process
pub struct VerificationOptions {
    pub timeout: Duration,
    pub proof_level: ProofLevel,
    pub resource_limits: ResourceLimits,
}
