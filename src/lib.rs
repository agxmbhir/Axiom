pub mod models;
pub mod traits;
pub mod errors;
pub mod config;
pub mod implementations;
#[cfg(test)]
pub mod tests;

// Re-export core components
pub use config::{AxiomConfig, AxiomOptions};
pub use errors::{AxiomError, AxiomResult, ErrorContext, ErrorSeverity, RecoverableError};
pub use implementations::specification_generator::LLMSpecificationGenerator;
pub use models::{
    common::{
        Domain, 
        Language, 
        VerificationSystem,
        VerificationLanguage,
        SpecificationParadigm,
    },
    property::{
        Property, 
        PropertyKind,
    },
    specification::{
        Specification,
        FormalSpecification,
        SpecificationOptions,
    },
    implementation::Implementation, 
    verification::{
        VerificationResult, 
        VerificationStatus,
    },
    artifact::VerifiedArtifact,
};
pub use traits::{
    SpecificationGenerator,
    ValidationDepth,
    VerificationLanguageIntegration,
    ImplementationGenerator,
    VerificationEngine,
    VerificationBackendAdapter,
    ProofAssistant,
    VerificationComplexity,
    ProofDifficulty,
    AutomationLevel,
    LanguageAdapter,
    AxiomSystem,
};