use thiserror::Error;

/// Custom error types for the Axiom system
#[derive(Debug, Error)]
pub enum AxiomError {
    #[error("Specification error: {0}")]
    SpecificationError(String),
    
    #[error("Implementation error: {0}")]
    ImplementationError(String),
    
    #[error("Verification error: {0}")]
    VerificationError(String),
    
    #[error("System error: {0}")]
    SystemError(String),
    
    #[error("Error in external tool {tool}: {message}")]
    ExternalToolError { tool: String, message: String },
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Specification translation error: {0}")]
    SpecTranslationError(String),

    #[error("Formal language error for {language}: {message}")]
    FormalLanguageError { language: String, message: String },

    #[error("Type error in formal specification: {0}")]
    FormalTypeError(String),

    #[error("Proof error: {0}")]
    ProofError(String),

    #[error("Language compatibility error: {source_lang} cannot be verified with {target_lang}")]
    LanguageCompatibilityError { source_lang: String, target_lang: String },

    #[error("Failed to parse natural language requirement: {0}")]
    RequirementParsingError(String),

    #[error("Ambiguous requirement detected: {requirement}. Possible interpretations: {interpretations:?}")]
    AmbiguousRequirementError { requirement: String, interpretations: Vec<String> },

    #[error("Inconsistent specification: {0}")]
    InconsistentSpecificationError(String),

    #[error("Missing verification dependencies: {0}")]
    MissingDependenciesError(String),

    #[error("Failed to integrate with verification tool: {tool}, reason: {reason}")]
    VerificationToolIntegrationError { tool: String, reason: String },
}

/// Result type specific to Axiom operations
pub type AxiomResult<T> = Result<T, AxiomError>;

/// Context for error reporting
pub struct ErrorContext {
    pub source_location: Option<String>,
    pub related_requirement: Option<String>,
    pub stack_trace: Vec<String>,
    pub suggestion: Option<String>,
    pub severity: ErrorSeverity,
}

/// Error severity levels
pub enum ErrorSeverity {
    Fatal,
    Error,
    Warning,
    Info,
}

/// Recoverable vs. non-recoverable errors
pub trait RecoverableError {
    fn is_recoverable(&self) -> bool;
    fn recovery_strategy(&self) -> Option<String>;
}