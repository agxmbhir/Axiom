use std::collections::HashMap;
use std::time::Duration;
use crate::models::common::{Language, ResourceLimits, VerificationSystem};

/// Configuration for the Axiom system
pub struct AxiomConfig {
    pub verification_system: VerificationSystem,
    pub target_languages: Vec<Language>,
    pub resource_limits: ResourceLimits,
    pub external_tools_config: ExternalToolsConfig,
}

/// Configuration for external verification tools
pub struct ExternalToolsConfig {
    pub tool_paths: HashMap<String, String>,
    pub timeout: Duration,
}

/// Options for the main Axiom system
pub struct AxiomOptions {
    pub specification_options: crate::models::specification::SpecificationOptions,
    pub implementation_options: crate::models::implementation::ImplementationOptions,
    pub verification_options: crate::models::verification::VerificationOptions,
}