use anyhow::{ anyhow, Result };
use std::fs;
use std::path::Path;

use crate::cli::ui;
use crate::models::common::Domain;
use crate::models::specification::{ FormalSpecification, Specification, SpecificationMetadata };
use crate::traits::axiom_system::AxiomSystem;
use crate::traits::specification_generator::ValidationDepth;

/// Specification validation command
pub async fn execute<S: AxiomSystem>(
    axiom: &S,
    spec_path: &Path,
    depth_str: &str,
    requirements_path: Option<&Path>
) -> Result<()> {
    ui::print_header("Validating Formal Specification");

    // Parse validation depth
    let validation_depth = parse_validation_depth(depth_str)?;

    // Load specification
    ui::print_info("Loading specification...");
    let spec_content = match fs::read_to_string(spec_path) {
        Ok(content) => content,
        Err(e) => {
            return Err(anyhow!("Failed to read specification file: {}", e));
        }
    };

    // Parse file extension to determine the verification language
    let verification_language = spec_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            match ext {
                "fst" => crate::models::common::VerificationLanguage::FStarLang,
                "dfy" => crate::models::common::VerificationLanguage::DafnyLang,
                "v" => crate::models::common::VerificationLanguage::CoqLang,
                "thy" => crate::models::common::VerificationLanguage::IsabelleLang,
                "lean" => crate::models::common::VerificationLanguage::LeanLang,
                "tla" => crate::models::common::VerificationLanguage::TLAPlus,
                "why" => crate::models::common::VerificationLanguage::Why3Lang,
                "smt2" => crate::models::common::VerificationLanguage::Z3SMT,
                _ => crate::models::common::VerificationLanguage::Custom(ext.to_string()),
            }
        })
        .unwrap_or(crate::models::common::VerificationLanguage::FStarLang);

    // Load requirements if provided
    let requirements = if let Some(req_path) = requirements_path {
        match fs::read_to_string(req_path) {
            Ok(content) =>
                content
                    .lines()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<String>>(),
            Err(e) => {
                return Err(anyhow!("Failed to read requirements file: {}", e));
            }
        }
    } else {
        vec!["Specification validation".to_string()]
    };

    // Create a formal specification struct
    let formal_spec = FormalSpecification {
        verification_language: verification_language.clone(),
        spec_code: spec_content.clone(),
        components: std::collections::HashMap::new(),
        dependencies: vec![],
    };

    // Create a full specification struct
    let spec = Specification {
        id: "validation_spec".to_string(),
        source_requirements: requirements.clone(),
        formal_properties: vec![],
        formal_spec,
        metadata: SpecificationMetadata {
            created_at: chrono::Utc::now(),
            verification_system: match verification_language {
                crate::models::common::VerificationLanguage::FStarLang =>
                    crate::models::common::VerificationSystem::FStar,
                crate::models::common::VerificationLanguage::DafnyLang =>
                    crate::models::common::VerificationSystem::Dafny,
                crate::models::common::VerificationLanguage::CoqLang =>
                    crate::models::common::VerificationSystem::Coq,
                crate::models::common::VerificationLanguage::IsabelleLang =>
                    crate::models::common::VerificationSystem::Isabelle,
                crate::models::common::VerificationLanguage::LeanLang =>
                    crate::models::common::VerificationSystem::Lean,
                crate::models::common::VerificationLanguage::TLAPlus =>
                    crate::models::common::VerificationSystem::TLA,
                crate::models::common::VerificationLanguage::Why3Lang =>
                    crate::models::common::VerificationSystem::Why3,
                crate::models::common::VerificationLanguage::Z3SMT =>
                    crate::models::common::VerificationSystem::Z3,
                _ =>
                    crate::models::common::VerificationSystem::Custom(
                        verification_language.to_string()
                    ),
            },
            domain: Domain::Custom("validation".to_string()),
            confidence_score: 0.9,
            is_formally_validated: false,
        },
    };

    // Perform validation
    ui::print_info(format!("Validating with {} depth...", depth_str).as_str());
    let spinner = ui::spinner_with_message("Validating specification...");

    // Validate the specification
    let validation_result = match
        axiom.validate_specification(&spec, &requirements, validation_depth)
    {
        Ok(is_valid) => {
            spinner.finish_with_message("Validation completed!");
            is_valid
        }
        Err(e) => {
            spinner.finish_with_message("Validation failed!");
            return Err(anyhow!("Validation error: {}", e));
        }
    };

    // Display validation result
    if validation_result {
        ui::print_success("Specification is valid!");

        // If requirements are provided, check completeness
        if requirements_path.is_some() {
            match axiom.verify_specification_completeness(&spec, &requirements).await {
                Ok((is_complete, missing)) => {
                    if is_complete {
                        ui::print_success("Specification completely covers all requirements!");
                    } else {
                        ui::print_warning("Specification does not cover all requirements.");
                        ui::print_info("Missing requirements:");
                        for (i, req) in missing.iter().enumerate() {
                            ui::print_info(format!("  {}. {}", i + 1, req).as_str());
                        }
                    }
                }
                Err(e) => {
                    ui::print_warning(format!("Could not check completeness: {}", e).as_str());
                }
            }
        }

        // Generate and display natural language description
        ui::print_info("Generating natural language description...");
        match generate_description(&spec).await {
            Ok(description) => {
                ui::print_header("Specification Description");
                // ui::print_text(&description);
            }
            Err(e) => {
                ui::print_warning(format!("Could not generate description: {}", e).as_str());
            }
        }
    } else {
        ui::print_error("Specification is invalid!");
    }

    ui::print_success("Validation completed!");

    Ok(())
}

fn parse_validation_depth(depth_str: &str) -> Result<ValidationDepth> {
    match depth_str.to_lowercase().as_str() {
        "basic" => Ok(ValidationDepth::Basic),
        "typecheck" => Ok(ValidationDepth::TypeCheck),
        "formal" => Ok(ValidationDepth::FormalVerification),
        _ => Err(anyhow!("Unsupported validation depth: {}", depth_str)),
    }
}

// Function to generate a natural language description of the specification
async fn generate_description(spec: &Specification) -> Result<String> {
    // Generate a description based on the specification
    // Since we can't reliably call LLM APIs from within another async context, we'll use
    // a template-based approach for now

    // Extract key information from the specification
    let language = &spec.formal_spec.verification_language;
    let code = &spec.formal_spec.spec_code;
    let requirements = &spec.source_requirements;

    // Extract functions/methods/theorems from the code
    let functions = extract_functions(code, language);
    let types = extract_types(code, language);

    // Create the description
    let type_count = types.len();
    let function_count = functions.len();

    let type_section = if !types.is_empty() {
        format!(
            "### Types\n\n{}\n\n",
            types
                .iter()
                .map(|t| format!("- `{}`", t))
                .collect::<Vec<_>>()
                .join("\n")
        )
    } else {
        "".to_string()
    };

    let function_section = if !functions.is_empty() {
        format!(
            "### Functions and Properties\n\n{}\n",
            functions
                .iter()
                .map(|f| format!("- `{}`", f))
                .collect::<Vec<_>>()
                .join("\n")
        )
    } else {
        "".to_string()
    };

    // Create the description
    Ok(
        format!(
            "# Specification Overview\n\n\
        This is a {} specification that addresses the following requirements:\n\n\
        {}\n\n\
        ## Key Components\n\n\
        The specification includes {} types and {} functions/properties.\n\n\
        {}{}\
        This specification can be used as a basis for implementation and formal verification.",
            language,
            requirements
                .iter()
                .map(|r| format!("- {}", r))
                .collect::<Vec<_>>()
                .join("\n"),
            type_count,
            function_count,
            type_section,
            function_section
        )
    )
}

// Helper function to extract function/method/theorem names from the specification code
fn extract_functions(
    code: &str,
    language: &crate::models::common::VerificationLanguage
) -> Vec<String> {
    let mut functions = Vec::new();

    // Use different patterns based on the verification language
    let patterns = match language {
        crate::models::common::VerificationLanguage::FStarLang => vec!["val", "let"],
        crate::models::common::VerificationLanguage::DafnyLang =>
            vec!["method", "function", "predicate"],
        crate::models::common::VerificationLanguage::CoqLang =>
            vec!["Theorem", "Lemma", "Definition"],
        crate::models::common::VerificationLanguage::IsabelleLang =>
            vec!["theorem", "lemma", "definition"],
        _ => vec![],
    };

    if patterns.is_empty() {
        return functions;
    }

    // Extract function names using simple pattern matching
    for line in code.lines() {
        let trimmed = line.trim();
        for pattern in &patterns {
            if trimmed.starts_with(pattern) {
                // Extract the function name
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() > 1 {
                    let name = parts[1].trim_end_matches(':').trim_end_matches('{');
                    functions.push(name.to_string());
                }
            }
        }
    }

    functions
}

// Helper function to extract type definitions from the specification code
fn extract_types(
    code: &str,
    language: &crate::models::common::VerificationLanguage
) -> Vec<String> {
    let mut types = Vec::new();

    // Use different patterns based on the verification language
    let patterns = match language {
        crate::models::common::VerificationLanguage::FStarLang => vec!["type"],
        crate::models::common::VerificationLanguage::DafnyLang => vec!["class", "datatype", "type"],
        crate::models::common::VerificationLanguage::CoqLang =>
            vec!["Inductive", "Record", "Structure"],
        crate::models::common::VerificationLanguage::IsabelleLang =>
            vec!["datatype", "record", "type_synonym"],
        _ => vec![],
    };

    if patterns.is_empty() {
        return types;
    }

    // Extract type names using simple pattern matching
    for line in code.lines() {
        let trimmed = line.trim();
        for pattern in &patterns {
            if trimmed.starts_with(pattern) {
                // Extract the type name
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() > 1 {
                    let name = parts[1].trim_end_matches('=').trim_end_matches('{');
                    types.push(name.to_string());
                }
            }
        }
    }

    types
}
