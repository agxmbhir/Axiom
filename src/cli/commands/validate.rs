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
    requirements_path: Option<&Path>,
    is_project: bool
) -> Result<()> {
    ui::print_header("Validating Formal Specification");

    // Parse validation depth
    let validation_depth = parse_validation_depth(depth_str)?;

    // Determine actual spec path and requirements path
    let (actual_spec_path, actual_req_path) = if is_project {
        // We're validating a project in the projects directory
        let project_name = spec_path.to_string_lossy();
        let project_dir = Path::new("projects").join(&*project_name);

        if !project_dir.exists() {
            return Err(anyhow!("Project {} not found in projects directory", project_name));
        }

        // Look for a specification file in the project directory
        let mut found_spec_path = None;
        for ext in &["fst", "dfy", "v", "thy", "lean", "tla", "why", "smt2"] {
            let potential_path = project_dir.join(format!("spec.{}", ext));
            if potential_path.exists() {
                found_spec_path = Some(potential_path);
                break;
            }
        }

        let spec_file_path = found_spec_path.ok_or_else(||
            anyhow!("No specification file found in project {}", project_name)
        )?;

        // Look for requirements file
        let req_file_path = project_dir.join("requirements.txt");
        let req_path = if req_file_path.exists() {
            Some(req_file_path)
        } else {
            requirements_path.map(|p| p.to_path_buf())
        };

        ui::print_info(&format!("Using project: {}", project_name));
        ui::print_info(&format!("Using specification: {}", spec_file_path.display()));
        if let Some(ref p) = req_path {
            ui::print_info(&format!("Using requirements: {}", p.display()));
        }

        (spec_file_path, req_path)
    } else {
        // Standard validation of a direct specification file
        (spec_path.to_path_buf(), requirements_path.map(|p| p.to_path_buf()))
    };

    // Load specification
    ui::print_info("Loading specification...");
    let spec_content = match fs::read_to_string(&actual_spec_path) {
        Ok(content) => content,
        Err(e) => {
            return Err(anyhow!("Failed to read specification file: {}", e));
        }
    };

    // Always use F* as the verification language, regardless of file extension
    let verification_language = crate::models::common::VerificationLanguage::FStarLang;
    
    // Log a note if the file doesn't have a .fst extension
    let file_ext = actual_spec_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
        
    if file_ext != "fst" {
        ui::print_warning("Note: Using F* for verification regardless of file extension.");
    }

    // Load requirements if provided
    let requirements = if let Some(req_path) = actual_req_path {
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

    // Validate the specification - this now returns a ValidationReport instead of just a boolean
    let validation_report = match
        axiom.validate_specification(&spec, &requirements, validation_depth)
    {
        Ok(report) => {
            spinner.finish_with_message("Validation completed!");
            report
        }
        Err(e) => {
            spinner.finish_with_message("Validation failed!");
            return Err(anyhow!("Validation error: {}", e));
        }
    };

    // Check if auto-fixing was performed
    let mut fixed_spec_code = None;
    for issue in &validation_report.issues {
        if issue.message.contains("automatically fixed") {
            if let Some(fix) = &issue.suggested_fix {
                fixed_spec_code = Some(fix.clone());
                ui::print_success(&format!("{}", issue.message));
            }
        }
    }

    // Display validation result
    if validation_report.is_valid {
        ui::print_success("Specification is valid!");
        
        // If the specification was fixed, save the fixed version
        if let Some(fixed_code) = fixed_spec_code {
            ui::print_info("Saving the automatically fixed specification...");
            
            // Determine the output path
            let output_path = if is_project {
                // If it's a project, update the original file
                actual_spec_path.clone()
            } else {
                // Otherwise, create a new file with "_fixed" suffix
                let mut fixed_path = actual_spec_path.clone();
                let stem = fixed_path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                let ext = fixed_path.extension().unwrap_or_default().to_string_lossy().to_string();
                fixed_path.set_file_name(format!("{}_fixed.{}", stem, ext));
                fixed_path
            };
            
            // Write the fixed code to the file
            match std::fs::write(&output_path, &fixed_code) {
                Ok(_) => {
                    ui::print_success(&format!("Fixed specification saved to {}", output_path.display()));
                }
                Err(e) => {
                    ui::print_error(&format!("Failed to save fixed specification: {}", e));
                }
            }
        }

        // If requirements are provided, check completeness
        if !requirements.is_empty() {
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

        // Try to load the description file if we're validating a project
        if is_project {
            let project_name = spec_path.to_string_lossy();
            let project_dir = Path::new("projects").join(&*project_name);
            let description_path = project_dir.join("description.md");

            if description_path.exists() {
                match fs::read_to_string(&description_path) {
                    Ok(description) => {
                        ui::print_header("Specification Description");
                        ui::print_text(&description);
                    }
                    Err(e) => {
                        ui::print_warning(
                            format!("Failed to read description file: {}", e).as_str()
                        );
                    }
                }
            } else {
                // Generate a description if we don't have one
                ui::print_info("Generating natural language description...");
                match generate_description(&spec).await {
                    Ok(description) => {
                        ui::print_header("Specification Description");
                        ui::print_text(&description);

                        // Save the generated description to the project
                        if let Err(e) = fs::write(&description_path, &description) {
                            ui::print_warning(
                                format!("Failed to save description to file: {}", e).as_str()
                            );
                        } else {
                            ui::print_success(
                                format!(
                                    "Saved description to {}",
                                    description_path.display()
                                ).as_str()
                            );
                        }
                    }
                    Err(e) => {
                        ui::print_warning(
                            format!("Could not generate description: {}", e).as_str()
                        );
                    }
                }
            }
        } else {
            // Standard description generation for non-project validation
            ui::print_info("Generating natural language description...");
            match generate_description(&spec).await {
                Ok(description) => {
                    ui::print_header("Specification Description");
                    ui::print_text(&description);
                }
                Err(e) => {
                    ui::print_warning(format!("Could not generate description: {}", e).as_str());
                }
            }
        }
    } else {
        ui::print_error("Specification is invalid!");
        
        // Display issues found during validation
        ui::print_info("Issues found during validation:");
        for (i, issue) in validation_report.issues.iter().enumerate() {
            let severity_str = match issue.severity {
                crate::models::specification::IssueSeverity::Error => "ERROR",
                crate::models::specification::IssueSeverity::Warning => "WARNING",
                crate::models::specification::IssueSeverity::Info => "INFO",
            };
            
            if issue.message.contains("automatically fixed") || 
               issue.message.contains("Automatic fixing was attempted") {
                // Already displayed above
                continue;
            }
            
            let location = if let Some(line) = issue.line_number {
                format!("Line {}", line)
            } else {
                "Unknown location".to_string()
            };
            
            ui::print_info(&format!("{}. [{}] {}: {}", 
                i + 1, 
                severity_str, 
                location,
                issue.message
            ));
            
            if let Some(fix) = &issue.suggested_fix {
                if fix.lines().count() < 6 {
                    // Only show short fixes inline
                    ui::print_info(&format!("   Suggested fix: {}", fix));
                } else {
                    ui::print_info("   Suggested fix available (see validation report)");
                }
            }
        }
        
        // Check if there's a best effort fix available to display
        if let Some(fix_issue) = validation_report.issues.iter().find(|i| 
            i.message.contains("Automatic fixing was attempted") && i.suggested_fix.is_some()
        ) {
            ui::print_warning("Automatic fixing was attempted but could not resolve all issues.");
            ui::print_info("Would you like to save the best effort fixed specification? (y/n)");
            
            // Simple user prompt for saving best effort fix
            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_ok() {
                if input.trim().to_lowercase() == "y" {
                    if let Some(fixed_code) = &fix_issue.suggested_fix {
                        // Determine the output path
                        let output_path = if is_project {
                            let mut fixed_path = actual_spec_path.clone();
                            fixed_path.set_file_name("spec_best_effort_fix.fst");
                            fixed_path
                        } else {
                            let mut fixed_path = actual_spec_path.clone();
                            let stem = fixed_path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                            let ext = fixed_path.extension().unwrap_or_default().to_string_lossy().to_string();
                            fixed_path.set_file_name(format!("{}_best_effort_fix.{}", stem, ext));
                            fixed_path
                        };
                        
                        // Write the best effort fixed code to the file
                        match std::fs::write(&output_path, fixed_code) {
                            Ok(_) => {
                                ui::print_success(&format!("Best effort fix saved to {}", output_path.display()));
                            }
                            Err(e) => {
                                ui::print_error(&format!("Failed to save best effort fix: {}", e));
                            }
                        }
                    }
                }
            }
        }
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
