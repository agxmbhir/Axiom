/// Supported formal verification systems
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationSystem {
    FStar,
    Dafny,
    Coq,
    Isabelle,
    Lean,
    TLA,
    Why3,
    Z3,
    Custom(String),
}

/// Formal verification specification languages
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationLanguage {
    FStarLang,
    DafnyLang,
    CoqLang,
    IsabelleLang,
    LeanLang,
    TLAPlus,
    Why3Lang,
    Z3SMT,
    ACSL,         // For C verification
    JML,          // For Java verification
    Liquid,       // For Haskell verification
    RustMIRAI,    // For Rust verification
    Custom(String),
}

/// Application domains for verification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Domain {
    Cryptography,
    DistributedSystems,
    WebSecurity,
    MachineLearning,
    SystemsSoftware,
    Blockchain,
    SafetyControl,
    HighAssuranceSoftware,
    Custom(String),
}

/// Target implementation language
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Language {
    Rust,
    C,
    CPlusPlus,
    Python,
    JavaScript,
    Go,
    Haskell,
    OCaml,
    Java,
    CSharp,
    Scala,
    Swift,
    Custom(String),
}

/// Levels of proof strength
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofLevel {
    Quick,         // Fast but less thorough
    Standard,      // Balance between thoroughness and speed
    Thorough,      // Most complete verification
    Exhaustive,    // Highest assurance level
    Custom(String),
}

/// Optimization levels for implementation generation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptimizationLevel {
    None,
    Speed,
    Size,
    Security,      // Prioritize security properties
    Readability,   // Prioritize human readability
    Custom(String),
}

/// Resource usage during verification
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub memory_kb: u64,
    pub cpu_seconds: f64,
    pub peak_memory_kb: u64,
    pub lemmas_proven: usize,
}

/// Resource limits for verification
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_memory_kb: u64,
    pub max_cpu_seconds: u64,
    pub max_verification_time: std::time::Duration,
    pub max_proof_depth: Option<usize>,
    pub parallel_jobs: Option<usize>,
}

/// Maps between verification languages and implementation languages
#[derive(Debug, Clone)]
pub struct LanguageMapping {
    pub verification_language: VerificationLanguage,
    pub implementation_language: Language,
    pub compatibility_score: f32,
    pub requires_adapter: bool,
}

/// Formal specification paradigm
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecificationParadigm {
    PrePostConditions,
    TypeTheoretic,
    ModelChecking,
    TemporalLogic,
    Refinement,
    HoareLogic,
    SeparationLogic,
    Custom(String),
}

/// Features of a verification language
#[derive(Debug, Clone)]
pub struct VerificationLanguageFeatures {
    pub language: VerificationLanguage,
    pub paradigm: SpecificationParadigm,
    pub supports_inductive_proofs: bool,
    pub supports_dependent_types: bool,
    pub supports_refinement_types: bool,
    pub has_automated_tactics: bool,
    pub has_smt_integration: bool,
}