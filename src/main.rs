use std::path::PathBuf;
use axiom::SpecificationGenerator;
use clap::Parser;
use log::{ error, info };
use anyhow::Result;
use axiom;
use crate::axiom::models;
use crate::axiom::config;
use crate::axiom::errors;
use crate::axiom::traits;
use crate::axiom::implementations::specification_generator::LLMSpecificationGenerator;
use crate::axiom::implementations::config::GeneratorConfig;
mod cli;
use cli::{ AxiomCli, Commands };

// Implementation of AxiomSystem that uses the LLMSpecificationGenerator for specification functionality
struct AxiomSystemImpl {
    spec_generator: LLMSpecificationGenerator,
}

impl AxiomSystemImpl {
    fn new() -> Self {
        // Create a default config for the specification generator
        let config = GeneratorConfig::default();
        let spec_generator = LLMSpecificationGenerator::new(config);

        Self { spec_generator }
    }
}

impl crate::axiom::traits::axiom_system::AxiomSystem for AxiomSystemImpl {
    fn process_requirements(
        &self,
        _requirements: &[String],
        _target_language: crate::models::common::Language,
        _domain: crate::models::common::Domain,
        _options: &crate::config::AxiomOptions
    ) -> crate::errors::AxiomResult<crate::models::artifact::VerifiedArtifact> {
        Err(crate::errors::AxiomError::SystemError("Not implemented".to_string()))
    }

    fn verify_existing_implementation(
        &self,
        _source_code: &str,
        _requirements: &[String],
        _language: crate::models::common::Language,
        _domain: crate::models::common::Domain
    ) -> crate::errors::AxiomResult<crate::models::verification::VerificationResult> {
        Err(crate::errors::AxiomError::SystemError("Not implemented".to_string()))
    }

    fn refine_to_satisfy(
        &self,
        _implementation: &crate::models::implementation::Implementation,
        _spec: &crate::models::specification::Specification
    ) -> crate::errors::AxiomResult<crate::models::implementation::Implementation> {
        Err(crate::errors::AxiomError::SystemError("Not implemented".to_string()))
    }

    // Generate a formal specification from requirements
    fn generate_formal_specification(
        &self,
        requirements: &[String],
        domain: crate::models::common::Domain,
        verification_language: crate::models::common::VerificationLanguage,
        options: &crate::models::specification::SpecificationOptions
    ) -> crate::errors::AxiomResult<crate::models::specification::FormalSpecification> {
        info!("Generating formal specification for domain: {:?}", domain);
        info!("Verification language: {:?}", verification_language);
        info!("Requirements count: {}", requirements.len());
        
        // Log the requirements
        info!("Requirements:");
        for (i, req) in requirements.iter().enumerate() {
            info!("  Requirement {}: {}", i+1, req);
        }
        
        // Instead of trying to use block_on inside an async context,
        // we'll create a separate runtime for synchronous use within this function
        
        // Clone the values we need to move into the closure
        let requirements_clone = requirements.to_vec();
        let domain_clone = domain.clone();
        let options_clone = options.clone();
        let generator = self.spec_generator.clone();
        
        // Spawn a new thread with a new runtime to handle the async call
        info!("Creating new thread to handle async specification generation");
        let handle = std::thread::spawn(move || {
            // Create a new runtime for this thread
            let rt = tokio::runtime::Runtime::new().unwrap();
            
            // Execute the async task in the new runtime
            rt.block_on(async {
                let mut spec_options = options_clone;
                spec_options.verification_language = verification_language.clone();
                
                // Use the generator to create the specification
                info!("Calling LLM API for specification generation");
                let spec = generator.generate_specification(
                    &requirements_clone,
                    domain_clone,
                    &spec_options
                ).await?;
                
                // Return just the formal specification part
                Ok::<_, crate::errors::AxiomError>(spec.formal_spec)
            })
        });
        
        // Wait for the thread to complete and get the result
        info!("Waiting for specification generation to complete");
        match handle.join() {
            Ok(result) => {
                match result {
                    Ok(formal_spec) => {
                        info!("Successfully generated specification with {} characters", 
                              formal_spec.spec_code.len());
                        Ok(formal_spec)
                    },
                    Err(e) => {
                        error!("Error generating specification: {}", e);
                        Err(e)
                    }
                }
            },
            Err(e) => {
                error!("Thread panicked during specification generation");
                Err(crate::errors::AxiomError::SystemError(
                    format!("Thread panic during specification generation: {:?}", e)
                ))
            }
        }
    }

    // Validate a specification
    fn validate_specification(
        &self,
        spec: &crate::models::specification::Specification,
        requirements: &[String],
        validation_depth: crate::traits::specification_generator::ValidationDepth
    ) -> crate::errors::AxiomResult<bool> {
        info!("Validating specification with depth: {:?}", validation_depth);
        
        // Clone the values we need to move into the closure
        let spec_clone = spec.clone();
        let validation_depth_clone = validation_depth.clone();
        let generator = self.spec_generator.clone();
        
        // Spawn a new thread with a new runtime to handle the async call
        info!("Creating new thread to handle async specification validation");
        let handle = std::thread::spawn(move || {
            // Create a new runtime for this thread
            let rt = tokio::runtime::Runtime::new().unwrap();
            
            // Execute the async task in the new runtime
            rt.block_on(async {
                info!("Calling LLM API for specification validation");
                let validation_report = generator.validate_specification(
                    &spec_clone,
                    validation_depth_clone
                ).await?;
                
                if !validation_report.is_valid {
                    info!("Validation failed with {} issues", validation_report.issues.len());
                    for (i, issue) in validation_report.issues.iter().enumerate() {
                        info!("Issue {}: {} (severity: {:?})", i + 1, issue.message, issue.severity);
                        if let Some(line) = issue.line_number {
                            info!("  At line: {}", line);
                        }
                        if let Some(fix) = &issue.suggested_fix {
                            info!("  Suggested fix: {}", fix);
                        }
                    }
                } else {
                    info!("Validation successful");
                }
                
                Ok::<_, crate::errors::AxiomError>(validation_report.is_valid)
            })
        });
        
        // Wait for the thread to complete and get the result
        info!("Waiting for validation to complete");
        match handle.join() {
            Ok(result) => {
                match result {
                    Ok(is_valid) => {
                        info!("Validation completed, result: {}", if is_valid { "valid" } else { "invalid" });
                        Ok(is_valid)
                    },
                    Err(e) => {
                        error!("Error during validation: {}", e);
                        Err(e)
                    }
                }
            },
            Err(e) => {
                error!("Thread panicked during validation");
                Err(crate::errors::AxiomError::SystemError(
                    format!("Thread panic during validation: {:?}", e)
                ))
            }
        }
    }

    fn generate_implementation_from_formal_spec(
        &self,
        _formal_spec: &crate::models::specification::FormalSpecification,
        _target_language: crate::models::common::Language,
        _options: &crate::models::implementation::ImplementationOptions
    ) -> crate::errors::AxiomResult<crate::models::implementation::Implementation> {
        Err(crate::errors::AxiomError::SystemError("Not implemented".to_string()))
    }

    fn verify_against_formal_spec(
        &self,
        _implementation: &crate::models::implementation::Implementation,
        _formal_spec: &crate::models::specification::FormalSpecification,
        _options: &crate::models::verification::VerificationOptions
    ) -> crate::errors::AxiomResult<crate::models::verification::VerificationResult> {
        Err(crate::errors::AxiomError::SystemError("Not implemented".to_string()))
    }

    fn is_verification_system_available(
        &self,
        _system: crate::models::common::VerificationSystem
    ) -> crate::errors::AxiomResult<bool> {
        // Assume all verification systems are available for this prototype
        Ok(true)
    }

    fn get_recommended_verification_system(
        &self,
        domain: crate::models::common::Domain,
        _implementation_language: crate::models::common::Language
    ) -> crate::errors::AxiomResult<crate::models::common::VerificationSystem> {
        // Return a recommended system based on the domain
        match domain {
            crate::models::common::Domain::Cryptography =>
                Ok(crate::models::common::VerificationSystem::FStar),
            crate::models::common::Domain::DistributedSystems =>
                Ok(crate::models::common::VerificationSystem::TLA),
            crate::models::common::Domain::WebSecurity =>
                Ok(crate::models::common::VerificationSystem::Dafny),
            _ => Ok(crate::models::common::VerificationSystem::FStar),
        }
    }

    fn export_verification_project(
        &self,
        _artifact: &crate::models::artifact::VerifiedArtifact,
        _output_dir: &std::path::Path,
        _system: crate::models::common::VerificationSystem
    ) -> crate::errors::AxiomResult<()> {
        Err(crate::errors::AxiomError::SystemError("Not implemented".to_string()))
    }

    fn import_verification_results(
        &self,
        _project_dir: &std::path::Path,
        _system: crate::models::common::VerificationSystem
    ) -> crate::errors::AxiomResult<crate::models::verification::VerificationResult> {
        Err(crate::errors::AxiomError::SystemError("Not implemented".to_string()))
    }

    fn get_error_context(
        &self,
        _verification_result: &crate::models::verification::VerificationResult,
        _implementation: &crate::models::implementation::Implementation,
        _spec: &crate::models::specification::Specification
    ) -> crate::errors::ErrorContext {
        crate::errors::ErrorContext {
            source_location: None,
            related_requirement: None,
            stack_trace: vec![],
            suggestion: None,
            severity: crate::errors::ErrorSeverity::Error,
        }
    }

    fn translate_verification_language(
        &self,
        _spec: &crate::models::specification::FormalSpecification,
        _target_language: crate::models::common::VerificationLanguage
    ) -> crate::errors::AxiomResult<crate::models::specification::FormalSpecification> {
        Err(crate::errors::AxiomError::SystemError("Not implemented".to_string()))
    }

    // Method to check if a specification completely covers the requirements
    async fn verify_specification_completeness(
        &self,
        spec: &crate::models::specification::Specification,
        requirements: &[String]
    ) -> crate::errors::AxiomResult<(bool, Vec<String>)> {
        info!("Checking specification completeness against {} requirements", requirements.len());
        
        // Call the generator to verify completeness
        info!("Calling LLM to verify specification completeness");
        let result = self.spec_generator.verify_specification_completeness(spec, requirements).await?;
        
        if result.0 {
            info!("Specification covers all requirements");
        } else {
            info!("Specification does not cover all requirements");
            info!("Missing requirements:");
            for (i, req) in result.1.iter().enumerate() {
                info!("  Missing requirement {}: {}", i + 1, req);
            }
        }
        
        Ok(result)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse the command line arguments
    let cli = AxiomCli::parse();

    // Setup logging
    setup_logging(&cli.log_level);

    // Create an instance of the Axiom system using our implementation
    let axiom_system = AxiomSystemImpl::new();

    // Display a welcome message
    println!("Axiom - AI-generated Code Verification System");

    // Handle commands
    match &cli.command {
        Commands::Process {
            requirements,
            language,
            domain,
            output,
            system,
            verification_language,
            interactive,
        } => {
            // Parse implementation language
            let lang = match language.to_lowercase().as_str() {
                "rust" => crate::models::common::Language::Rust,
                "c" => crate::models::common::Language::C,
                "c++" | "cpp" => crate::models::common::Language::CPlusPlus,
                "python" | "py" => crate::models::common::Language::Python,
                "javascript" | "js" => crate::models::common::Language::JavaScript,
                "go" => crate::models::common::Language::Go,
                "haskell" | "hs" => crate::models::common::Language::Haskell,
                "ocaml" | "ml" => crate::models::common::Language::OCaml,
                "java" => crate::models::common::Language::Java,
                "csharp" | "c#" => crate::models::common::Language::CSharp,
                "scala" => crate::models::common::Language::Scala,
                "swift" => crate::models::common::Language::Swift,
                _ => crate::models::common::Language::Custom(language.clone()),
            };

            // Parse domain
            let dom = match domain.to_lowercase().as_str() {
                "crypto" | "cryptography" => crate::models::common::Domain::Cryptography,
                "distributed" | "distributedsystems" =>
                    crate::models::common::Domain::DistributedSystems,
                "web" | "websecurity" => crate::models::common::Domain::WebSecurity,
                "ml" | "machinelearning" => crate::models::common::Domain::MachineLearning,
                "systems" | "systemssoftware" => crate::models::common::Domain::SystemsSoftware,
                "blockchain" => crate::models::common::Domain::Blockchain,
                "safety" | "safetycontrol" => crate::models::common::Domain::SafetyControl,
                "highassurance" => crate::models::common::Domain::HighAssuranceSoftware,
                _ => crate::models::common::Domain::Custom(domain.clone()),
            };

            // Parse verification system if provided
            let verification_sys = match system {
                Some(sys) =>
                    match sys.to_lowercase().as_str() {
                        "fstar" | "f*" => Some(crate::models::common::VerificationSystem::FStar),
                        "dafny" => Some(crate::models::common::VerificationSystem::Dafny),
                        "coq" => Some(crate::models::common::VerificationSystem::Coq),
                        "isabelle" => Some(crate::models::common::VerificationSystem::Isabelle),
                        "lean" => Some(crate::models::common::VerificationSystem::Lean),
                        "tla" | "tla+" => Some(crate::models::common::VerificationSystem::TLA),
                        "why3" => Some(crate::models::common::VerificationSystem::Why3),
                        "z3" => Some(crate::models::common::VerificationSystem::Z3),
                        _ => Some(crate::models::common::VerificationSystem::Custom(sys.clone())),
                    }
                None => None,
            };

            // Parse verification language if provided
            let verification_lang = match verification_language {
                Some(lang) =>
                    match lang.to_lowercase().as_str() {
                        "fstar" => Some(crate::models::common::VerificationLanguage::FStarLang),
                        "dafny" => Some(crate::models::common::VerificationLanguage::DafnyLang),
                        "coq" => Some(crate::models::common::VerificationLanguage::CoqLang),
                        "isabelle" =>
                            Some(crate::models::common::VerificationLanguage::IsabelleLang),
                        "lean" => Some(crate::models::common::VerificationLanguage::LeanLang),
                        "tla" | "tlaplus" =>
                            Some(crate::models::common::VerificationLanguage::TLAPlus),
                        "why3" => Some(crate::models::common::VerificationLanguage::Why3Lang),
                        "z3" | "smt" => Some(crate::models::common::VerificationLanguage::Z3SMT),
                        "acsl" => Some(crate::models::common::VerificationLanguage::ACSL),
                        "jml" => Some(crate::models::common::VerificationLanguage::JML),
                        "liquid" => Some(crate::models::common::VerificationLanguage::Liquid),
                        "mirai" => Some(crate::models::common::VerificationLanguage::RustMIRAI),
                        _ =>
                            Some(crate::models::common::VerificationLanguage::Custom(lang.clone())),
                    }
                None => None,
            };

            // Execute the process command
            cli::commands::process::execute(
                &axiom_system,
                requirements,
                lang,
                dom,
                output,
                verification_sys,
                verification_lang,
                *interactive
            ).await?;
        }

        // Spec command - generate a formal specification
        Commands::Spec { requirements, verification_language, domain, output, detail_level } => {
            cli::commands::spec::execute(
                &axiom_system,
                requirements,
                verification_language,
                domain,
                output.as_deref(), // Convert Option<PathBuf> to Option<&Path>
                detail_level
            ).await?;
        }

        // Validate command - validate a formal specification
        Commands::Validate { spec, depth, requirements } => {
            cli::commands::validate::execute(
                &axiom_system,
                spec,
                depth,
                requirements.as_deref()
            ).await?;
        }

        // Other commands are not yet implemented
        _ => {
            cli::ui::print_info("Command not yet implemented.");
            cli::ui::print_info(
                "This is a prototype CLI interface. Only the 'spec', 'validate', and 'process' commands are implemented."
            );
        }
    }

    Ok(())
}

fn setup_logging(log_level: &str) {
    // Set up the logger based on the log level
    let level = match log_level.to_lowercase().as_str() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info,
    };

    env_logger::Builder::new().filter_level(level).init();

    info!("Logger initialized with level: {}", log_level);
}