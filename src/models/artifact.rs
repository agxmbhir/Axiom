use crate::models::implementation::Implementation;
use crate::models::specification::Specification;
use crate::models::verification::VerificationResult;

/// Final output of the Axiom system
pub struct VerifiedArtifact {
    pub requirements: Vec<String>,
    pub specification: Specification,
    pub implementation: Implementation,
    pub verification_result: VerificationResult,
    pub documentation: Documentation,
}

/// Documentation for verified artifacts
pub struct Documentation {
    pub spec_explanation: String,
    pub impl_explanation: String,
    pub verification_summary: String,
    pub usage_examples: Vec<String>,
}