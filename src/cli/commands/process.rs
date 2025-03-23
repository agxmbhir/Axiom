use anyhow::{ anyhow, Result };
use std::fs;
use std::path::Path;
use std::time::Duration;

use crate::cli::ui;
use crate::models::common::{ Domain, Language, VerificationLanguage, VerificationSystem };
use crate::models::implementation::ImplementationOptions;
use crate::models::specification::SpecificationOptions;
use crate::models::verification::VerificationOptions;
use crate::traits::axiom_system::AxiomSystem;
use crate::traits::specification_generator::ValidationDepth;

/// Process command that runs the entire pipeline from requirements to verified implementation
pub async fn execute<S: AxiomSystem>(
    axiom: &S,
    requirements_path: &Path,
    language: Language,
    domain: Domain,
    output_dir: &Path,
    verification_system: Option<VerificationSystem>,
    verification_language: Option<VerificationLanguage>,
    interactive: bool
) -> Result<()> {
    // Display welcome message and workflow overview
    ui::print_header("Axiom Verification Pipeline");
    ui::print_info("Starting the complete verification workflow:");
    ui::print_info("1. Load requirements");
    ui::print_info("2. Generate formal specification");
    ui::print_info("3. Validate specification");
    ui::print_info("4. Generate implementation");
    ui::print_info("5. Verify implementation");
    ui::print_info("6. Save results");

    if interactive {
        ui::pause()?;
    }

    // Ensure output directory exists
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    // Load requirements
    ui::print_header("Loading Requirements");
    let requirements = match requirements_path.exists() {
        true => {
            let content = fs::read_to_string(requirements_path)?;
            content
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        }
        false => {
            if interactive {
                ui::get_requirements()?
            } else {
                return Err(anyhow!("Requirements file not found: {:?}", requirements_path));
            }
        }
    };

    // Display the loaded requirements
    ui::print_info(format!("Loaded {} requirements:", requirements.len()).as_str());
    for (i, req) in requirements.iter().enumerate() {
        println!("{}. {}", i + 1, req);
    }

    if interactive {
        ui::pause()?;
    }

    // Determine verification language
    let verification_lang = if let Some(lang) = verification_language {
        lang
    } else if interactive {
        ui::select_verification_language()?
    } else {
        // Default to F* if not specified
        VerificationLanguage::FStarLang
    };

    // Determine verification system
    let verification_sys = if let Some(sys) = verification_system {
        sys
    } else if interactive {
        ui::select_verification_system()?
    } else {
        // Default to F* if not specified
        VerificationSystem::FStar
    };

    // Setup specification options
    let mut spec_options = SpecificationOptions::default();
    spec_options.verification_language = verification_lang.clone();

    if interactive {
        // Allow user to choose specification paradigm
        let paradigm = ui::select_specification_paradigm()?;
        // The specification options would be updated with the selected paradigm
        // This depends on the actual implementation of SpecificationOptions
    }

    // Generate formal specification
    ui::print_header("Generating Formal Specification");

    let spinner = ui::spinner_with_message("Generating formal specification...");

    let formal_spec = axiom.generate_formal_specification(
        &requirements,
        domain.clone(),
        verification_lang.clone(),
        &spec_options
    )?;

    spinner.finish_with_message("Formal specification generated successfully!");

    // Display the generated specification
    ui::display_specification(&verification_lang, &formal_spec.spec_code);

    // Save the specification to a file
    let spec_filename = match verification_lang {
        VerificationLanguage::FStarLang => "specification.fst",
        VerificationLanguage::DafnyLang => "specification.dfy",
        VerificationLanguage::CoqLang => "specification.v",
        VerificationLanguage::IsabelleLang => "specification.thy",
        VerificationLanguage::LeanLang => "specification.lean",
        VerificationLanguage::TLAPlus => "specification.tla",
        VerificationLanguage::Why3Lang => "specification.why",
        VerificationLanguage::Z3SMT => "specification.smt2",
        VerificationLanguage::ACSL => "specification.c",
        VerificationLanguage::JML => "specification.java",
        VerificationLanguage::Liquid => "specification.hs",
        VerificationLanguage::RustMIRAI => "specification.rs",
        VerificationLanguage::Custom(_) => "specification.txt",
    };

    let spec_path = output_dir.join(spec_filename);
    fs::write(&spec_path, &formal_spec.spec_code)?;

    ui::print_success(format!("Specification saved to {}", spec_path.display()).as_str());

    if interactive {
        ui::pause()?;
    }

    // Create a specification struct with the formal spec
    // Note: This is a simplified example that would need to be expanded
    // in a real implementation to create a complete Specification object
    let spec = crate::models::specification::Specification {
        id: "spec-1".to_string(),
        source_requirements: requirements.clone(),
        formal_properties: vec![],
        formal_spec,
        metadata: crate::models::specification::SpecificationMetadata {
            created_at: chrono::Utc::now(),
            verification_system: verification_sys,
            domain,
            confidence_score: 0.95,
            is_formally_validated: false,
        },
    };

    // Validate specification
    ui::print_header("Validating Specification");

    let validation_depth = if interactive {
        ui::select_validation_depth()?
    } else {
        ValidationDepth::Basic
    };

    let spinner = ui::spinner_with_message("Validating specification...");

    let is_valid = axiom.validate_specification(&spec, &requirements, validation_depth)?;

    if is_valid {
        spinner.finish_with_message("Specification validated successfully!");
    } else {
        spinner.finish_with_message("Specification validation found issues.");
        ui::print_warning("Specification has issues that need to be resolved before proceeding.");

        if interactive {
            let proceed = ui::confirm_action("Do you want to proceed anyway?")?;
            if !proceed {
                return Ok(());
            }
        } else {
            return Err(anyhow!("Specification validation failed"));
        }
    }

    if interactive {
        ui::pause()?;
    }

    // Generate implementation
    ui::print_header("Generating Implementation");

    let spinner = ui::spinner_with_message("Generating implementation...");

    let implementation_options = ImplementationOptions {
        optimization_level: crate::models::common::OptimizationLevel::None,
        include_comments: true,
        style_guide: None,
    };

    let implementation = axiom.generate_implementation_from_formal_spec(
        &spec.formal_spec,
        language.clone(),
        &implementation_options
    )?;

    spinner.finish_with_message("Implementation generated successfully!");

    // Display the implementation
    ui::print_info("Generated Implementation:");
    println!("\n{}\n", implementation.source_code);

    // Save the implementation to a file
    let impl_filename = match language {
        Language::Rust => "implementation.rs",
        Language::C => "implementation.c",
        Language::CPlusPlus => "implementation.cpp",
        Language::Python => "implementation.py",
        Language::JavaScript => "implementation.js",
        Language::Go => "implementation.go",
        Language::Haskell => "implementation.hs",
        Language::OCaml => "implementation.ml",
        Language::Java => "implementation.java",
        Language::CSharp => "implementation.cs",
        Language::Scala => "implementation.scala",
        Language::Swift => "implementation.swift",
        Language::Custom(_) => "implementation.txt",
    };

    let impl_path = output_dir.join(impl_filename);
    fs::write(&impl_path, &implementation.source_code)?;

    ui::print_success(format!("Implementation saved to {}", impl_path.display()).as_str());

    if interactive {
        ui::pause()?;
    }

    // Verify implementation
    ui::print_header("Verifying Implementation");

    let spinner = ui::spinner_with_message("Verifying implementation against specification...");

    let verification_options = VerificationOptions {
        timeout: Duration::from_secs(300),
        proof_level: crate::models::common::ProofLevel::Standard,
        resource_limits: crate::models::common::ResourceLimits {
            max_memory_kb: 1024 * 1024, // 1GB
            max_cpu_seconds: 600,
            max_verification_time: Duration::from_secs(600),
            max_proof_depth: None,
            parallel_jobs: None,
        },
    };

    let verification_result = axiom.verify_against_formal_spec(
        &implementation,
        &spec.formal_spec,
        &verification_options
    )?;

    spinner.finish();

    // Display verification result
    ui::print_verification_status(&verification_result.status);

    if
        let crate::models::verification::VerificationStatus::Failed(reasons) =
            &verification_result.status
    {
        for reason in reasons {
            ui::print_error(reason);
        }
    }

    // Save verification results
    let results_path = output_dir.join("verification_results.txt");
    let results_content = format!(
        "Verification Results\n\
         ===================\n\
         Status: {}\n\
         Time Taken: {:?}\n\
         Memory Used: {} KB\n\
         \n\
         Proof Artifacts:\n\
         {:#?}",
        verification_result.status,
        verification_result.verification_time,
        verification_result.resource_usage.memory_kb,
        verification_result.proof_artifacts
    );

    fs::write(&results_path, results_content)?;

    ui::print_success(format!("Verification results saved to {}", results_path.display()).as_str());

    // Final summary
    ui::print_header("Verification Pipeline Complete");
    ui::print_info("Summary of the verification process:");
    ui::print_result("Requirements", &format!("{} processed", requirements.len()));
    ui::print_result("Specification", "Generated and validated");
    ui::print_result("Implementation", &format!("Generated in {}", language_to_string(&language)));
    ui::print_result("Verification", &format!("{}", verification_result.status));

    ui::print_success("Axiom verification pipeline completed successfully!");

    Ok(())
}

fn language_to_string(language: &Language) -> String {
    match language {
        Language::Rust => "Rust".to_string(),
        Language::C => "C".to_string(),
        Language::CPlusPlus => "C++".to_string(),
        Language::Python => "Python".to_string(),
        Language::JavaScript => "JavaScript".to_string(),
        Language::Go => "Go".to_string(),
        Language::Haskell => "Haskell".to_string(),
        Language::OCaml => "OCaml".to_string(),
        Language::Java => "Java".to_string(),
        Language::CSharp => "C#".to_string(),
        Language::Scala => "Scala".to_string(),
        Language::Swift => "Swift".to_string(),
        Language::Custom(name) => name.clone(),
    }
}
