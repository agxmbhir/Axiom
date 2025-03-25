use clap::{ Parser, Subcommand };
use std::path::PathBuf;

pub mod commands;
pub mod ui;

#[derive(Parser)]
#[command(
    name = "axiom",
    about = "A system that formally verifies AI-generated code",
    version,
    author,
    long_about = None
)]
pub struct AxiomCli {
    /// Sets the log level (error, warn, info, debug, trace)
    #[arg(short, long, global = true, default_value = "info")]
    pub log_level: String,

    /// Path to configuration file
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    /// Output format (text, json)
    #[arg(long, global = true, default_value = "text")]
    pub output_format: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new Axiom project
    Init {
        /// Project directory
        #[arg(default_value = ".")]
        directory: PathBuf,

        /// Target implementation language
        #[arg(short, long)]
        language: Option<String>,

        /// Verification system to use
        #[arg(short, long)]
        verification_system: Option<String>,

        /// Application domain
        #[arg(short, long)]
        domain: Option<String>,
    },

    /// Generate a formal specification from requirements
    Spec {
        /// Path to requirements file (one requirement per line)
        #[arg(short, long)]
        requirements: PathBuf,

        /// Verification language to generate
        #[arg(short, long, default_value = "fstar")]
        verification_language: String,

        /// Domain for the specification
        #[arg(short, long)]
        domain: String,

        /// Output file for the specification
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Detailed level for specification generation
        #[arg(long, default_value = "standard")]
        detail_level: String,
    },

    /// Validate a formal specification
    Validate {
        /// Path to specification file or project name in projects directory
        #[arg(short, long)]
        spec: PathBuf,

        /// Validation depth (basic, typecheck, formal)
        #[arg(short, long, default_value = "basic")]
        depth: String,

        /// Requirements file for completeness checking
        #[arg(short, long)]
        requirements: Option<PathBuf>,
        
        /// Validate a project in the projects directory
        #[arg(short, long, default_value = "false")]
        project: bool,
    },

    /// Generate implementation from a specification
    Implement {
        /// Path to specification file
        #[arg(short, long)]
        spec: PathBuf,

        /// Target language for implementation
        #[arg(short, long)]
        language: String,

        /// Optimization level (none, speed, size, security, readability)
        #[arg(short, long, default_value = "none")]
        optimization: String,

        /// Output file for implementation
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Include implementation comments
        #[arg(long, default_value = "true")]
        comments: bool,
    },

    /// Verify an implementation against a specification
    Verify {
        /// Path to implementation file
        #[arg(short, long)]
        implementation: PathBuf,

        /// Path to specification file
        #[arg(short, long)]
        spec: PathBuf,

        /// Verification system to use
        #[arg(short, long)]
        system: Option<String>,

        /// Output directory for verification results
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Proof level (quick, standard, thorough, exhaustive)
        #[arg(short, long, default_value = "standard")]
        proof_level: String,

        /// Timeout in seconds
        #[arg(short, long, default_value = "300")]
        timeout: u64,
    },

    /// Process requirements through the entire pipeline
    Process {
        /// Path to requirements file
        #[arg(short, long)]
        requirements: PathBuf,

        /// Target implementation language
        #[arg(short, long)]
        language: String,

        /// Domain for the specification
        #[arg(short, long)]
        domain: String,

        /// Output directory for all generated files
        #[arg(short, long)]
        output: PathBuf,

        /// Verification system to use
        #[arg(short, long)]
        system: Option<String>,

        /// Verification language to use
        #[arg(long)]
        verification_language: Option<String>,

        /// Interactive mode
        #[arg(short, long, default_value = "true")]
        interactive: bool,
    },

    /// Translate between verification languages
    Translate {
        /// Path to source specification file
        #[arg(short, long)]
        source: PathBuf,

        /// Target verification language
        #[arg(short, long)]
        target_language: String,

        /// Output file for translated specification
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// List supported languages, verification systems, and domains
    List {
        /// What to list (languages, verification-systems, domains, verification-languages)
        #[arg(default_value = "all")]
        what: String,
    },

    /// Check tool availability and integration
    Check {
        /// Verification system to check
        #[arg(short, long)]
        system: Option<String>,

        /// Implementation language to check
        #[arg(short, long)]
        language: Option<String>,

        /// Install missing dependencies if possible
        #[arg(short, long, default_value = "false")]
        install: bool,
    },
}
