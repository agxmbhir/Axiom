pub mod common;
pub mod property;
pub mod specification;
pub mod implementation;
pub mod verification;
pub mod artifact;

// Re-export common model types
pub use common::{Domain, Language, VerificationSystem};
pub use property::{Property, PropertyKind};
pub use specification::Specification;
pub use implementation::Implementation;
pub use verification::{VerificationResult, VerificationStatus};
pub use artifact::{VerifiedArtifact, Documentation};