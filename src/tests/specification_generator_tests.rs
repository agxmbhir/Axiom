#[cfg(test)]
mod tests {
    use std::env;
    use dotenv::dotenv;
    use log::{ debug, info, warn, error };

    use tokio::test;

    use crate::errors::{ AxiomError, AxiomResult };
    use crate::implementations::config::GeneratorConfig;
    use crate::implementations::specification_generator::LLMSpecificationGenerator;
    use crate::models::common::{ Domain, VerificationLanguage };
    use crate::models::specification::SpecificationOptions;
    use crate::traits::specification_generator::{ SpecificationGenerator, ValidationDepth };

    // Setup function to initialize logging and environment
    fn setup() {
        // Initialize logger if not already initialized
        match env_logger::try_init() {
            Ok(_) => {
                info!("Logger initialized");
            }
            Err(_) => {
                // Logger already initialized, which is fine
            }
        }

        // Try to load .env file, but don't fail if it doesn't exist
        match dotenv() {
            Ok(_) => {
                debug!("Loaded environment variables from .env file");
            }
            Err(e) => {
                warn!("Could not load .env file: {}", e);
                info!("Will try to use environment variables that are already set");
            }
        }
    }

    // Check if any API key is available
    fn should_skip_api_tests() -> bool {
        setup(); // Ensure environment is loaded

        let api_keys = vec![
            "ANTHROPIC_API_KEY",
            "OPENAI_API_KEY",
            "MISTRAL_API_KEY",
            "TOGETHER_API_KEY",
            "AZURE_OPENAI_API_KEY"
        ];

        let any_key_available = api_keys.iter().any(|key| env::var(key).is_ok());

        if !any_key_available {
            warn!("No API keys found. Skipping tests that require API access.");
        } else {
            info!("Found at least one API key. Tests requiring API access will run.");
        }

        !any_key_available
    }

    fn get_test_requirements() -> Vec<String> {
        debug!("Loading test requirements");
        vec![
            "The system must encrypt all user data at rest using AES-256".to_string(),
            "Encryption keys must be rotated every 90 days".to_string(),
            "All encryption operations must be resistant to timing attacks".to_string()
        ]
    }

    fn create_test_generator() -> LLMSpecificationGenerator {
        info!("Creating test generator with default configuration");
        let config = GeneratorConfig::default();
        LLMSpecificationGenerator::new(config)
    }

    // This is our main test for specification generation
    #[test]
    #[ignore = "Requires API key"]
    async fn test_generate_specification() -> AxiomResult<()> {
        if should_skip_api_tests() {
            info!("Skipping test_generate_specification that requires API key");
            return Ok(());
        }

        info!("Running test_generate_specification");
        let generator: LLMSpecificationGenerator = create_test_generator();

        info!("Getting test requirements");
        let requirements = get_test_requirements();
        debug!("Requirements: {:?}", requirements);

        let domain = Domain::Cryptography;
        info!("Using domain: {:?}", domain);

        info!("Setting up specification options");
        let mut options = SpecificationOptions::default();
        options.verification_language = VerificationLanguage::FStarLang;
        debug!("Verification language: {:?}", options.verification_language);

        info!("Calling generate_specification");
        let spec = match generator.generate_specification(&requirements, domain, &options).await {
            Ok(s) => {
                info!("Successfully generated specification");
                s
            }
            Err(e) => {
                error!("Failed to generate specification: {}", e);
                return Err(e);
            }
        };

        info!("Verifying generated specification structure");
        // Verify the basic structure of the generated specification
        debug!("Specification ID: {}", spec.id);
        assert!(!spec.id.is_empty(), "Specification ID should not be empty");

        debug!("Verifying requirements match");
        assert_eq!(
            spec.source_requirements,
            requirements,
            "Source requirements should match input requirements"
        );

        debug!("Verifying verification language");
        assert_eq!(
            spec.formal_spec.verification_language,
            VerificationLanguage::FStarLang,
            "Verification language should be F*"
        );

        debug!("Specification code length: {} characters", spec.formal_spec.spec_code.len());
        assert!(!spec.formal_spec.spec_code.is_empty(), "Specification code should not be empty");

        info!("Checking for F* specific syntax in generated code");
        // The generated code should contain F* specific syntax for encryption
        let has_encrypt_let = spec.formal_spec.spec_code.contains("let encrypt");
        let has_encrypt_val = spec.formal_spec.spec_code.contains("val encrypt");
        let has_key_type = spec.formal_spec.spec_code.contains("type key");

        debug!("Contains 'let encrypt': {}", has_encrypt_let);
        debug!("Contains 'val encrypt': {}", has_encrypt_val);
        debug!("Contains 'type key': {}", has_key_type);

        assert!(
            has_encrypt_let || has_encrypt_val || has_key_type,
            "Generated code should contain F* specific syntax for encryption"
        );

        // Write the generated specification to a file in the projects directory for inspection
        let project_dir = std::path::Path::new("projects").join("test_crypto_project");
        if let Err(e) = std::fs::create_dir_all(&project_dir) {
            warn!("Could not create project directory: {}", e);
        }

        let spec_file_path = project_dir.join("specification.fst");
        if let Err(e) = std::fs::write(&spec_file_path, &spec.formal_spec.spec_code) {
            warn!("Could not write specification to file: {}", e);
        } else {
            info!("Specification written to {}", spec_file_path.display());
        }

        info!("test_generate_specification completed successfully");
        Ok(())
    }

    // Simple test for basic validation
    #[test]
    #[ignore = "Requires API key"]
    async fn test_validate_specification() -> AxiomResult<()> {
        if should_skip_api_tests() {
            info!("Skipping test_validate_specification that requires API key");
            return Ok(());
        }

        info!("Running test_validate_specification");
        let generator = create_test_generator();

        info!("Getting test requirements");
        let requirements = get_test_requirements();

        let domain = Domain::Cryptography;
        info!("Using domain: {:?}", domain);

        info!("Setting up specification options");
        let mut options = SpecificationOptions::default();
        options.verification_language = VerificationLanguage::FStarLang;

        info!("Generating specification for validation");
        let spec = match generator.generate_specification(&requirements, domain, &options).await {
            Ok(s) => {
                info!("Successfully generated specification");
                s
            }
            Err(e) => {
                error!("Failed to generate specification: {}", e);
                return Err(e);
            }
        };

        info!("Validating specification with basic validation depth");
        // Basic validation should usually pass for generated specs
        let validation = match
            generator.validate_specification(&spec, ValidationDepth::Basic).await
        {
            Ok(v) => {
                info!("Successfully validated specification");
                v
            }
            Err(e) => {
                error!("Failed to validate specification: {}", e);
                return Err(e);
            }
        };

        debug!("Validation result - is_valid: {}", validation.is_valid);
        debug!("Validation issues count: {}", validation.issues.len());

        // Log any validation issues
        if !validation.issues.is_empty() {
            warn!("Validation found {} issues:", validation.issues.len());
            for (i, issue) in validation.issues.iter().enumerate() {
                warn!("Issue {}: {} (severity: {:?})", i + 1, issue.message, issue.severity);
                if let Some(line) = issue.line_number {
                    debug!("  At line: {}", line);
                }
                if let Some(fix) = &issue.suggested_fix {
                    debug!("  Suggested fix: {}", fix);
                }
            }
        }

        info!("test_validate_specification completed successfully");
        Ok(())
    }

    // Test the mock validator to ensure it works without API calls
    #[test]
    async fn test_mock_validation() {
        info!("Running test_mock_validation");

        info!("Creating mock specification");
        // Create a mock specification
        let mock_spec = crate::models::specification::Specification {
            id: "mock_spec".to_string(),
            source_requirements: vec!["Test requirement".to_string()],
            formal_properties: vec![],
            formal_spec: crate::models::specification::FormalSpecification {
                verification_language: VerificationLanguage::FStarLang,
                spec_code: "module Test\nlet test (x:int) : int = x + 1".to_string(),
                components: std::collections::HashMap::new(),
                dependencies: vec![],
            },
            metadata: crate::models::specification::SpecificationMetadata {
                created_at: chrono::Utc::now(),
                verification_system: crate::models::common::VerificationSystem::FStar,
                domain: Domain::Cryptography,
                confidence_score: 0.9,
                is_formally_validated: false,
            },
        };

        debug!("Mock specification ID: {}", mock_spec.id);
        debug!("Mock specification language: {:?}", mock_spec.formal_spec.verification_language);
        debug!("Mock code: {}", mock_spec.formal_spec.spec_code);

        info!("Creating test generator for error context");
        let generator = create_test_generator();

        info!("Getting error context from generator");
        // Create a mock error context
        let error_ctx = generator.get_error_context("Test error", &mock_spec);

        if let Some(suggestion) = &error_ctx.suggestion {
            debug!("Error suggestion: {}", suggestion);
        } else {
            debug!("No error suggestion provided");
        }

        info!("Verifying error context");
        // Verify the error context has appropriate severity
        assert!(
            matches!(error_ctx.severity, crate::errors::ErrorSeverity::Error),
            "Error context should have Error severity"
        );
        assert!(error_ctx.suggestion.is_some(), "Error context should have a suggestion");

        info!("test_mock_validation completed successfully");
    }
}
