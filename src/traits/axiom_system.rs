use std::path::Path;
use crate::config::AxiomOptions;
use crate::errors::{ AxiomResult, ErrorContext };
use crate::models::artifact::VerifiedArtifact;
use crate::models::common::{ Domain, Language, VerificationLanguage, VerificationSystem };
use crate::models::implementation::{ Implementation, ImplementationOptions };
use crate::models::specification::{ Specification, SpecificationOptions, FormalSpecification };
use crate::models::verification::{ VerificationResult, VerificationOptions };
use crate::traits::specification_generator::ValidationDepth;

/// Main facade for the Axiom system
pub trait AxiomSystem {
    /// Process natural language requirements through the entire pipeline
    fn process_requirements(
        &self,
        requirements: &[String],
        target_language: Language,
        domain: Domain,
        options: &AxiomOptions
    ) -> AxiomResult<VerifiedArtifact>;

    // Method to check if a specification completely covers the requirements
    async fn verify_specification_completeness(
        &self,
        spec: &crate::models::specification::Specification,
        requirements: &[String]
    ) -> crate::errors::AxiomResult<(bool, Vec<String>)>;

    /// Verify an existing implementation against requirements
    fn verify_existing_implementation(
        &self,
        source_code: &str,
        requirements: &[String],
        language: Language,
        domain: Domain
    ) -> AxiomResult<VerificationResult>;

    /// Refine an existing implementation to satisfy its specification
    fn refine_to_satisfy(
        &self,
        implementation: &Implementation,
        spec: &Specification
    ) -> AxiomResult<Implementation>;

    /// Generate formal specification from natural language requirements
    fn generate_formal_specification(
        &self,
        requirements: &[String],
        domain: Domain,
        verification_language: VerificationLanguage,
        options: &SpecificationOptions
    ) -> AxiomResult<FormalSpecification>;

    /// Validate a specification against requirements and check its internal consistency
    fn validate_specification(
        &self,
        spec: &Specification,
        requirements: &[String],
        validation_depth: ValidationDepth
    ) -> AxiomResult<bool>;

    /// Generate implementation from formal specification
    fn generate_implementation_from_formal_spec(
        &self,
        formal_spec: &FormalSpecification,
        target_language: Language,
        options: &ImplementationOptions
    ) -> AxiomResult<Implementation>;

    /// Verify implementation against formal specification
    fn verify_against_formal_spec(
        &self,
        implementation: &Implementation,
        formal_spec: &FormalSpecification,
        options: &VerificationOptions
    ) -> AxiomResult<VerificationResult>;

    /// Check if a particular verification system is supported and available
    fn is_verification_system_available(&self, system: VerificationSystem) -> AxiomResult<bool>;

    /// Get the most suitable verification system for a given domain
    fn get_recommended_verification_system(
        &self,
        domain: Domain,
        implementation_language: Language
    ) -> AxiomResult<VerificationSystem>;

    /// Export verification project files for external verification tools
    fn export_verification_project(
        &self,
        artifact: &VerifiedArtifact,
        output_dir: &Path,
        system: VerificationSystem
    ) -> AxiomResult<()>;

    /// Import verification results from an external verification tool
    fn import_verification_results(
        &self,
        project_dir: &Path,
        system: VerificationSystem
    ) -> AxiomResult<VerificationResult>;

    /// Get error context for a failed verification
    fn get_error_context(
        &self,
        verification_result: &VerificationResult,
        implementation: &Implementation,
        spec: &Specification
    ) -> ErrorContext;

    /// Translate between verification languages
    fn translate_verification_language(
        &self,
        spec: &FormalSpecification,
        target_language: VerificationLanguage
    ) -> AxiomResult<FormalSpecification>;
}
