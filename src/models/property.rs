/// A formal property that must be satisfied by an implementation
#[derive(Debug, Clone)]
pub struct Property {
    pub id: String,
    pub description: String,
    pub formal_definition: String,
    pub kind: PropertyKind,
}

/// Types of formal properties that can be verified
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyKind {
    Functional,       // Correct behavior
    Safety,           // Nothing bad happens
    Liveness,         // Something good eventually happens
    Security,         // Resistance to attacks
    ResourceUsage,    // Bounds on memory, time, etc.
    Custom(String),   // Domain-specific properties
}