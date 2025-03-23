use std::path::Path;
use async_trait::async_trait;

use crate::errors::{AxiomResult, ErrorContext};
use crate::models::common::{Domain, VerificationLanguage, SpecificationParadigm};
use crate::models::specification::{
    Specification, 
    ValidationReport, 
    SpecificationOptions, 
    SpecificationTranslation,
    FormalSpecification,
    VerificationTemplate
};

/// Main trait for translating natural language requirements to formal specifications
#[async_trait]
pub trait SpecificationGenerator {
    /// Generate a formal specification from natural language requirements
    async fn generate_specification(
        &self, 
        requirements: &[String], 
        domain: Domain,
        options: &SpecificationOptions,
    ) -> AxiomResult<Specification>;
    
    /// Refine an existing specification based on feedback
    async fn refine_specification(
        &self, 
        spec: &Specification, 
        feedback: &str,
        options: &SpecificationOptions,
    ) -> AxiomResult<Specification>;
    
    /// Check if a specification is complete and well-formed
    async fn validate_specification(
        &self, 
        spec: &Specification,
        validation_depth: ValidationDepth,
    ) -> AxiomResult<ValidationReport>;
    
    /// Translate natural language requirements to formal properties
    async fn translate_to_properties(
        &self,
        requirements: &[String],
        domain: Domain,
    ) -> AxiomResult<Vec<SpecificationTranslation>>;
    
    /// Convert properties to a formal specification in target verification language
    async fn convert_to_formal_specification(
        &self,
        translations: &[SpecificationTranslation],
        target_language: VerificationLanguage,
        paradigm: SpecificationParadigm,
    ) -> AxiomResult<FormalSpecification>;
    
    /// Translate a specification from one verification language to another
    async fn translate_specification(
        &self,
        spec: &Specification,
        target_language: VerificationLanguage,
    ) -> AxiomResult<Specification>;
    
    /// Check if the formal specification satisfies all requirements
    async fn verify_specification_completeness(
        &self,
        spec: &Specification,
        requirements: &[String],
    ) -> AxiomResult<(bool, Vec<String>)>;
    
    /// Generate executable verification code for a verification system
    async fn generate_verification_code(
        &self,
        spec: &Specification,
        target_system: crate::models::common::VerificationSystem,
    ) -> AxiomResult<String>;
    
    /// Retrieve available specification templates for a given domain and language
    async fn get_specification_templates(
        &self,
        domain: Domain,
        language: VerificationLanguage,
    ) -> AxiomResult<Vec<VerificationTemplate>>;
    
    /// Apply a template to generate a formal specification
    async fn apply_template(
        &self,
        template: &VerificationTemplate,
        properties: &[crate::models::property::Property],
    ) -> AxiomResult<FormalSpecification>;
    
    /// Export the specification to a file in the appropriate format for verification
    async fn export_specification(
        &self,
        spec: &Specification,
        output_path: &Path,
    ) -> AxiomResult<()>;
    
    /// Import a specification from an existing formal specification file
    async fn import_specification(
        &self,
        spec_file: &Path,
        language: VerificationLanguage,
    ) -> AxiomResult<Specification>;
    
    /// Extract error context from a specification error
    fn get_error_context(&self, error: &str, spec: &Specification) -> ErrorContext;
}

/// Enum to control validation depth
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationDepth {
    /// Basic syntax and consistency checking
    Basic,
    /// Type checking and internal consistency
    TypeCheck,
    /// Full formal validation with a verification tool
    FormalVerification,
}

/// Trait for integrating with verification language tools
pub trait VerificationLanguageIntegration {
    /// Get the supported verification language
    fn supported_language(&self) -> VerificationLanguage;
    
    /// Check if a piece of specification code is valid syntax
    fn validate_syntax(&self, code: &str) -> AxiomResult<bool>;
    
    /// Type check a formal specification
    fn type_check(&self, spec: &FormalSpecification) -> AxiomResult<bool>;
    
    /// Compile a formal specification to an intermediate format
    fn compile_specification(&self, spec: &FormalSpecification) -> AxiomResult<Vec<u8>>;
    
    /// Generate standard libraries and imports for a specification
    fn generate_preamble(&self, domain: Domain) -> AxiomResult<String>;
    
    /// Transform a general property into language-specific syntax
    fn property_to_syntax(
        &self, 
        property: &crate::models::property::Property,
    ) -> AxiomResult<String>;
}