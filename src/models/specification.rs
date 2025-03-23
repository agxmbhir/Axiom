use chrono;
use std::collections::HashMap;

use crate::models::common::{Domain, VerificationSystem, VerificationLanguage};
use crate::models::property::Property;

/// Represents a formal specification derived from natural language requirements
#[derive(Debug, Clone)]
pub struct Specification {
    pub id: String,
    pub source_requirements: Vec<String>,
    pub formal_properties: Vec<Property>,
    /// The complete formal specification in the target verification language
    pub formal_spec: FormalSpecification,
    pub metadata: SpecificationMetadata,
}

/// The formal specification in a verification language
#[derive(Debug, Clone)]
pub struct FormalSpecification {
    /// The verification language used for this specification
    pub verification_language: VerificationLanguage,
    /// The complete formal specification code
    pub spec_code: String,
    /// Individual named components of the specification (theorems, lemmas, etc.)
    pub components: HashMap<String, String>,
    /// Environment/imports needed for the specification
    pub dependencies: Vec<String>,
}

/// Metadata associated with a specification
#[derive(Debug, Clone)]
pub struct SpecificationMetadata {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub verification_system: VerificationSystem,
    pub domain: Domain,
    pub confidence_score: f32,
    /// Indicates if the specification has been validated by formal methods
    pub is_formally_validated: bool,
}

/// Validation report for specifications
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub issues: Vec<ValidationIssue>,
    /// True if the specification has been checked by a verification tool
    pub tool_validated: bool,
    /// Output from the verification tool if used
    pub tool_output: Option<String>,
}

/// Issues found during specification validation
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub message: String,
    pub related_property: Option<String>,
    /// Line number in formal specification if applicable
    pub line_number: Option<usize>,
    /// Suggested fix if available
    pub suggested_fix: Option<String>,
}

/// Severity levels for validation issues
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

/// Options for specification generation
#[derive(Debug, Clone)]
pub struct SpecificationOptions {
    pub detail_level: DetailLevel,
    pub include_security_properties: bool,
    pub include_performance_properties: bool,
    /// Target verification language
    pub verification_language: VerificationLanguage,
    /// Control which parts of the specification to generate
    pub generation_targets: SpecGenerationTargets,
    /// Additional options specific to the verification language
    pub language_specific_options: HashMap<String, String>,
}

impl Default for SpecificationOptions {
    fn default() -> Self {
        Self {
            detail_level: DetailLevel::Standard,
            include_security_properties: true,
            include_performance_properties: true,
            verification_language: VerificationLanguage::FStarLang,
            generation_targets: SpecGenerationTargets::default(),
            language_specific_options: HashMap::new(),
        }
    }
}

/// Controls which specification components to generate
#[derive(Debug, Clone)]
pub struct SpecGenerationTargets {
    pub generate_invariants: bool,
    pub generate_pre_post_conditions: bool,
    pub generate_type_constraints: bool,
    pub generate_security_proofs: bool,
    pub generate_inductive_proofs: bool,
}

impl Default for SpecGenerationTargets {
    fn default() -> Self {
        Self {
            generate_invariants: true,
            generate_pre_post_conditions: true,
            generate_type_constraints: true,
            generate_security_proofs: true,
            generate_inductive_proofs: true,
        }
    }
}

/// Level of detail in specifications
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetailLevel {
    Minimal,
    Standard,
    Comprehensive,
    Custom(String),
}

/// Type for tracking the translation of natural language to formal specifications
#[derive(Debug, Clone)]
pub struct SpecificationTranslation {
    pub requirement: String,
    pub interpreted_properties: Vec<String>,
    pub formal_representation: String,
    pub translation_confidence: f32,
    pub verification_language: VerificationLanguage,
    pub requires_human_review: bool,
}

/// Defines a template for verification code in a specific language
#[derive(Debug, Clone)]
pub struct VerificationTemplate {
    pub language: VerificationLanguage,
    pub template_name: String,
    pub template_code: String,
    pub placeholders: Vec<String>,
    pub documentation: String,
}