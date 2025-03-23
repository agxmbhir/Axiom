use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

use crate::cli::ui;
use crate::models::common::{Domain, VerificationLanguage, SpecificationParadigm};
use crate::models::specification::SpecificationOptions;
use crate::traits::axiom_system::AxiomSystem;
use crate::traits::specification_generator::ValidationDepth;

/// Specification generation command
pub async fn execute<S: AxiomSystem>(
    axiom: &S,
    requirements_path: &Path,
    verification_language_str: &str,
    domain_str: &str,
    output_path: Option<&Path>,
    detail_level: &str,
) -> Result<()> {
    ui::print_header("Generating Formal Specification");
    
    // Parse verification language
    let verification_language = parse_verification_language(verification_language_str)?;
    
    // Parse domain
    let domain = parse_domain(domain_str)?;
    
    // Load requirements
    ui::print_info("Loading requirements...");
    let requirements = match fs::read_to_string(requirements_path) {
        Ok(content) => content.lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>(),
        Err(e) => return Err(anyhow!("Failed to read requirements file: {}", e)),
    };
    
    ui::print_info(format!("Loaded {} requirements", requirements.len()).as_str());
    
    // Setup specification options
    let mut spec_options = SpecificationOptions::default();
    spec_options.verification_language = verification_language.clone();
    
    // Set detail level
    spec_options.detail_level = match detail_level.to_lowercase().as_str() {
        "minimal" => crate::models::specification::DetailLevel::Minimal,
        "standard" => crate::models::specification::DetailLevel::Standard,
        "comprehensive" => crate::models::specification::DetailLevel::Comprehensive,
        _ => crate::models::specification::DetailLevel::Standard,
    };
    
    // Generate specification
    let spinner = ui::spinner_with_message("Generating formal specification...");
    
    let formal_spec = axiom.generate_formal_specification(
        &requirements,
        domain,
        verification_language.clone(),
        &spec_options,
    )?;
    
    spinner.finish_with_message("Specification generated successfully!");
    
    // Display the specification
    ui::display_specification(&verification_language, &formal_spec.spec_code);
    
    // Create project directory structure
    let project_name = format!("project_{}", chrono::Utc::now().timestamp());
    let project_dir = std::path::Path::new("projects").join(&project_name);
    fs::create_dir_all(&project_dir)?;
    
    // Determine file extension based on verification language
    let extension = match verification_language {
        VerificationLanguage::FStarLang => "fst",
        VerificationLanguage::DafnyLang => "dfy",
        VerificationLanguage::CoqLang => "v",
        VerificationLanguage::IsabelleLang => "thy",
        VerificationLanguage::LeanLang => "lean",
        VerificationLanguage::TLAPlus => "tla",
        VerificationLanguage::Why3Lang => "why",
        VerificationLanguage::Z3SMT => "smt2",
        _ => "txt",
    };
    
    // Save specification to project directory
    let spec_path = project_dir.join(format!("spec.{}", extension));
    fs::write(&spec_path, &formal_spec.spec_code)?;
    ui::print_success(format!("Specification saved to {}", spec_path.display()).as_str());
    
    // Save to output path if provided
    if let Some(output_path) = output_path {
        fs::write(output_path, &formal_spec.spec_code)?;
        ui::print_success(format!("Specification also saved to {}", output_path.display()).as_str());
    }
    
    ui::print_result("Dependencies", &formal_spec.dependencies.join(", "));
    ui::print_result("Components", &format!("{} components defined", formal_spec.components.len()));
    
    ui::print_success("Formal specification generation completed!");
    
    Ok(())
}

fn parse_verification_language(language_str: &str) -> Result<VerificationLanguage> {
    match language_str.to_lowercase().as_str() {
        "fstar" => Ok(VerificationLanguage::FStarLang),
        "dafny" => Ok(VerificationLanguage::DafnyLang),
        "coq" => Ok(VerificationLanguage::CoqLang),
        "isabelle" => Ok(VerificationLanguage::IsabelleLang),
        "lean" => Ok(VerificationLanguage::LeanLang),
        "tla" | "tlaplus" => Ok(VerificationLanguage::TLAPlus),
        "why3" => Ok(VerificationLanguage::Why3Lang),
        "z3" | "smt" => Ok(VerificationLanguage::Z3SMT),
        "acsl" => Ok(VerificationLanguage::ACSL),
        "jml" => Ok(VerificationLanguage::JML),
        "liquid" => Ok(VerificationLanguage::Liquid),
        "mirai" => Ok(VerificationLanguage::RustMIRAI),
        _ => Err(anyhow!("Unsupported verification language: {}", language_str)),
    }
}

fn parse_domain(domain_str: &str) -> Result<Domain> {
    match domain_str.to_lowercase().as_str() {
        "crypto" | "cryptography" => Ok(Domain::Cryptography),
        "distributed" | "distributedsystems" => Ok(Domain::DistributedSystems),
        "web" | "websecurity" => Ok(Domain::WebSecurity),
        "ml" | "machinelearning" => Ok(Domain::MachineLearning),
        "systems" | "systemssoftware" => Ok(Domain::SystemsSoftware),
        "blockchain" => Ok(Domain::Blockchain),
        "safety" | "safetycontrol" => Ok(Domain::SafetyControl),
        "highassurance" => Ok(Domain::HighAssuranceSoftware),
        _ => Ok(Domain::Custom(domain_str.to_string())),
    }
}