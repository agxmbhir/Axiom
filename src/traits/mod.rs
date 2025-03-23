pub mod specification_generator;
pub mod implementation_generator;
pub mod verification_engine;
pub mod language_adapter;
pub mod axiom_system;

// Re-export traits
pub use specification_generator::{SpecificationGenerator, ValidationDepth, VerificationLanguageIntegration};
pub use implementation_generator::ImplementationGenerator;
pub use verification_engine::{
    VerificationEngine, 
    VerificationBackendAdapter, 
    ProofAssistant,
    VerificationComplexity,
    ProofDifficulty,
    AutomationLevel,
};
pub use language_adapter::LanguageAdapter;
pub use axiom_system::AxiomSystem;