use colored::*;
use console::Term;
use dialoguer::{ theme::ColorfulTheme, Confirm, Input, Select };
use indicatif::{ ProgressBar, ProgressStyle };
use std::time::Duration;
use textwrap::wrap;

use crate::models::common::{
    Domain,
    Language,
    VerificationLanguage,
    VerificationSystem,
    SpecificationParadigm,
};
use crate::models::verification::VerificationStatus;
use crate::traits::specification_generator::ValidationDepth;

/// UI theme for consistent appearance
pub fn get_theme() -> ColorfulTheme {
    ColorfulTheme::default()
}

/// Print a section header
pub fn print_header(title: &str) {
    let width = Term::stdout().size().1 as usize;
    let title = format!(" {} ", title);
    println!("\n{}\n", title.bold().white().on_blue());
}

/// Print text with proper wrapping
pub fn print_text(text: &str) {
    let width = Term::stdout().size().1 as usize;
    for line in text.lines() {
        if line.starts_with('#') {
            // Handle headers
            println!("{}", line.bold());
        } else if line.starts_with('-') {
            // Handle list items
            println!("{}", line);
        } else if line.starts_with("```") {
            // Handle code blocks
            println!("{}", line);
        } else {
            // Wrap normal text
            for wrapped_line in wrap(line, width.saturating_sub(10)) {
                println!("{}", wrapped_line);
            }
        }
    }
}

/// Print an error message
pub fn print_error(message: &str) {
    eprintln!("{} {}", "ERROR:".red().bold(), message);
}

/// Print a warning message
pub fn print_warning(message: &str) {
    println!("{} {}", "WARNING:".yellow().bold(), message);
}

/// Print a success message
pub fn print_success(message: &str) {
    println!("{} {}", "SUCCESS:".green().bold(), message);
}

/// Print information
pub fn print_info(message: &str) {
    println!("{} {}", "INFO:".blue().bold(), message);
}

/// Print a formatted result
pub fn print_result(label: &str, value: &str) {
    println!("{}: {}", label.bold(), value);
}

/// Create a new progress bar
pub fn create_progress_bar(length: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(length);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}"
            )
            .unwrap()
            .progress_chars("##-")
    );
    pb.set_message(message.to_string());
    pb
}

/// Print verification status with color
pub fn print_verification_status(status: &VerificationStatus) {
    let (status_str, color) = match status {
        VerificationStatus::Verified => ("✓ Verified".to_string(), "green"),
        VerificationStatus::Unverified => ("? Unverified".to_string(), "yellow"),
        VerificationStatus::Failed(reasons) => {
            let mut status = format!("✗ Failed");
            if !reasons.is_empty() {
                status = format!("{} - {}", status, reasons.join(", "));
            }
            (status, "red")
        }
        VerificationStatus::Timeout => ("⏱ Timeout".to_string(), "yellow"),
        VerificationStatus::Error(msg) => (format!("⚠ Error: {}", msg), "red"),
    };

    match color {
        "green" => println!("{}", status_str.green().bold()),
        "yellow" => println!("{}", status_str.yellow().bold()),
        "red" => println!("{}", status_str.red().bold()),
        _ => println!("{}", status_str),
    }
}

/// Interactive selection of a language
pub fn select_language() -> std::io::Result<Language> {
    let languages = vec![
        "Rust",
        "C",
        "C++",
        "Python",
        "JavaScript",
        "Go",
        "Haskell",
        "OCaml",
        "Java",
        "C#",
        "Scala",
        "Swift"
    ];

    let selection = Select::with_theme(&get_theme())
        .with_prompt("Select implementation language")
        .items(&languages)
        .default(0)
        .interact()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let lang = match selection {
        0 => Language::Rust,
        1 => Language::C,
        2 => Language::CPlusPlus,
        3 => Language::Python,
        4 => Language::JavaScript,
        5 => Language::Go,
        6 => Language::Haskell,
        7 => Language::OCaml,
        8 => Language::Java,
        9 => Language::CSharp,
        10 => Language::Scala,
        11 => Language::Swift,
        _ => Language::Custom(languages[selection].to_string()),
    };

    Ok(lang)
}

/// Interactive selection of a verification system
pub fn select_verification_system() -> std::io::Result<VerificationSystem> {
    let systems = vec!["F*", "Dafny", "Coq", "Isabelle", "Lean", "TLA+", "Why3", "Z3"];

    let selection = Select::with_theme(&get_theme())
        .with_prompt("Select verification system")
        .items(&systems)
        .default(0)
        .interact()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let system = match selection {
        0 => VerificationSystem::FStar,
        1 => VerificationSystem::Dafny,
        2 => VerificationSystem::Coq,
        3 => VerificationSystem::Isabelle,
        4 => VerificationSystem::Lean,
        5 => VerificationSystem::TLA,
        6 => VerificationSystem::Why3,
        7 => VerificationSystem::Z3,
        _ => VerificationSystem::Custom(systems[selection].to_string()),
    };

    Ok(system)
}

/// Interactive selection of a verification language
pub fn select_verification_language() -> std::io::Result<VerificationLanguage> {
    let languages = vec![
        "F*",
        "Dafny",
        "Coq",
        "Isabelle",
        "Lean",
        "TLA+",
        "Why3",
        "Z3 SMT",
        "ACSL (C)",
        "JML (Java)",
        "Liquid Haskell",
        "MIRAI (Rust)"
    ];

    let selection = Select::with_theme(&get_theme())
        .with_prompt("Select verification language")
        .items(&languages)
        .default(0)
        .interact()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let lang = match selection {
        0 => VerificationLanguage::FStarLang,
        1 => VerificationLanguage::DafnyLang,
        2 => VerificationLanguage::CoqLang,
        3 => VerificationLanguage::IsabelleLang,
        4 => VerificationLanguage::LeanLang,
        5 => VerificationLanguage::TLAPlus,
        6 => VerificationLanguage::Why3Lang,
        7 => VerificationLanguage::Z3SMT,
        8 => VerificationLanguage::ACSL,
        9 => VerificationLanguage::JML,
        10 => VerificationLanguage::Liquid,
        11 => VerificationLanguage::RustMIRAI,
        _ => VerificationLanguage::Custom(languages[selection].to_string()),
    };

    Ok(lang)
}

/// Interactive selection of a domain
pub fn select_domain() -> std::io::Result<Domain> {
    let domains = vec![
        "Cryptography",
        "Distributed Systems",
        "Web Security",
        "Machine Learning",
        "Systems Software",
        "Blockchain",
        "Safety Control",
        "High Assurance Software"
    ];

    let selection = Select::with_theme(&get_theme())
        .with_prompt("Select application domain")
        .items(&domains)
        .default(0)
        .interact()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let domain = match selection {
        0 => Domain::Cryptography,
        1 => Domain::DistributedSystems,
        2 => Domain::WebSecurity,
        3 => Domain::MachineLearning,
        4 => Domain::SystemsSoftware,
        5 => Domain::Blockchain,
        6 => Domain::SafetyControl,
        7 => Domain::HighAssuranceSoftware,
        _ => Domain::Custom(domains[selection].to_string()),
    };

    Ok(domain)
}

/// Interactive selection of a specification paradigm
pub fn select_specification_paradigm() -> std::io::Result<SpecificationParadigm> {
    let paradigms = vec![
        "Pre/Post Conditions",
        "Type-Theoretic",
        "Model Checking",
        "Temporal Logic",
        "Refinement Types",
        "Hoare Logic",
        "Separation Logic"
    ];

    let selection = Select::with_theme(&get_theme())
        .with_prompt("Select specification paradigm")
        .items(&paradigms)
        .default(0)
        .interact()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let paradigm = match selection {
        0 => SpecificationParadigm::PrePostConditions,
        1 => SpecificationParadigm::TypeTheoretic,
        2 => SpecificationParadigm::ModelChecking,
        3 => SpecificationParadigm::TemporalLogic,
        4 => SpecificationParadigm::Refinement,
        5 => SpecificationParadigm::HoareLogic,
        6 => SpecificationParadigm::SeparationLogic,
        _ => SpecificationParadigm::Custom(paradigms[selection].to_string()),
    };

    Ok(paradigm)
}

/// Interactive selection of validation depth
pub fn select_validation_depth() -> std::io::Result<ValidationDepth> {
    let depths = vec![
        "Basic (syntax checking)",
        "Type Check (internal consistency)",
        "Formal Verification (full validation)"
    ];

    let selection = Select::with_theme(&get_theme())
        .with_prompt("Select validation depth")
        .items(&depths)
        .default(0)
        .interact()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let depth = match selection {
        0 => ValidationDepth::Basic,
        1 => ValidationDepth::TypeCheck,
        2 => ValidationDepth::FormalVerification,
        _ => ValidationDepth::Basic,
    };

    Ok(depth)
}

/// Get a list of requirements from the user
pub fn get_requirements() -> std::io::Result<Vec<String>> {
    let mut requirements = Vec::new();

    println!("Enter requirements (one per line, empty line to finish):");
    loop {
        let req: String = Input::with_theme(&get_theme())
            .with_prompt(format!("Requirement {}", requirements.len() + 1))
            .allow_empty(true)
            .interact()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        if req.is_empty() {
            break;
        }

        requirements.push(req);
    }

    Ok(requirements)
}

/// Display a formal specification with syntax highlighting
pub fn display_specification(language: &VerificationLanguage, code: &str) {
    print_header("Formal Specification");

    // This is a simple display without syntax highlighting
    // In a real implementation, you would use a syntax highlighter appropriate for the language
    let lang_name = match language {
        VerificationLanguage::FStarLang => "F*",
        VerificationLanguage::DafnyLang => "Dafny",
        VerificationLanguage::CoqLang => "Coq",
        VerificationLanguage::IsabelleLang => "Isabelle",
        VerificationLanguage::LeanLang => "Lean",
        VerificationLanguage::TLAPlus => "TLA+",
        VerificationLanguage::Why3Lang => "Why3",
        VerificationLanguage::Z3SMT => "Z3 SMT",
        VerificationLanguage::ACSL => "ACSL",
        VerificationLanguage::JML => "JML",
        VerificationLanguage::Liquid => "Liquid Haskell",
        VerificationLanguage::RustMIRAI => "MIRAI",
        VerificationLanguage::Custom(s) => s,
    };

    println!("Language: {}", lang_name.cyan());
    println!("\n{}\n", code);
}

/// Confirm an action with the user
pub fn confirm_action(prompt: &str) -> std::io::Result<bool> {
    Confirm::with_theme(&get_theme())
        .with_prompt(prompt)
        .default(true)
        .interact()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

/// Display a spinner while waiting for an operation to complete
pub fn spinner_with_message(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner
}

pub fn pause() -> std::io::Result<()> {
    println!("\nPress Enter to continue...");
    let _input: String = Input::with_theme(&get_theme())
        .allow_empty(true)
        .interact()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    Ok(())
}
