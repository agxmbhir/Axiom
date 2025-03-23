use crate::errors::AxiomResult;
use crate::models::common::Language;
use crate::models::implementation::{Implementation, ImplementationOptions};
use crate::models::specification::Specification;
use crate::models::verification::VerificationResult;

/// Trait for generating implementations from specifications
pub trait ImplementationGenerator {
    /// Generate an implementation from a specification in the target language
    fn generate_implementation(
        &self, 
        spec: &Specification, 
        language: Language,
        options: &ImplementationOptions,
    ) -> AxiomResult<Implementation>;
    
    /// Refine an implementation based on verification feedback
    fn refine_implementation(
        &self,
        implementation: &Implementation,
        verification_result: &VerificationResult,
    ) -> AxiomResult<Implementation>;
    
    /// Check if the implementation matches the specification before formal verification
    fn validate_implementation(&self, implementation: &Implementation, spec: &Specification) -> AxiomResult<bool>;
}