use crate::models::common::Language;
use crate::models::verification::VerificationResult;

/// Represents a verified implementation
pub struct Implementation {
    pub id: String,
    pub specification_id: String,
    pub language: Language,
    pub source_code: String,
    pub verification_result: VerificationResult,
}

/// Options for implementation generation
pub struct ImplementationOptions {
    pub optimization_level: crate::models::common::OptimizationLevel,
    pub include_comments: bool,
    pub style_guide: Option<String>,
}