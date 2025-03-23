use crate::errors::AxiomResult;
use crate::models::common::Language;
use crate::models::property::Property;

/// Trait for language-specific adapters
pub trait LanguageAdapter {
    /// Get the target language
    fn language(&self) -> Language;
    
    /// Convert language-agnostic properties to language-specific constraints
    fn convert_properties(&self, properties: &[Property]) -> AxiomResult<String>;
    
    /// Generate language-specific test cases from properties
    fn generate_tests(&self, properties: &[Property]) -> AxiomResult<String>;
    
    /// Check if a source code adheres to language-specific requirements
    fn validate_source(&self, source: &str) -> AxiomResult<bool>;
}