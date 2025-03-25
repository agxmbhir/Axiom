use std::collections::HashMap;
use std::path::{ Path, PathBuf };
use async_trait::async_trait;
use log::{ debug, error, info, warn };
use serde::{ Deserialize, Serialize };
use thiserror::Error;

use crate::errors::{ AxiomError, AxiomResult, ErrorContext, ErrorSeverity };
use crate::implementations::config::{ ConfigError, GeneratorConfig };
use crate::models::common::{ Domain, SpecificationParadigm, VerificationLanguage };
use crate::models::property::Property;
use crate::models::specification::{
    FormalSpecification,
    Specification,
    SpecificationOptions,
    SpecificationTranslation,
    ValidationReport,
    ValidationIssue,
    VerificationTemplate,
    IssueSeverity,
};
use crate::traits::specification_generator::{ SpecificationGenerator, ValidationDepth };

#[derive(Debug, Error)]
pub enum SpecGenError {
    #[error("API error: {0}")] ApiError(String),

    #[error("Configuration error: {0}")] ConfigError(#[from] ConfigError),

    #[error("Failed to parse API response: {0}")] ParseError(String),

    #[error("Template error: {0}")] TemplateError(String),

    #[error("Validation error: {0}")] ValidationError(String),

    #[error("Network error: {0}")] NetworkError(String),

    #[error(transparent)] IoError(#[from] std::io::Error),

    #[error(transparent)] SerdeError(#[from] serde_json::Error),

    #[error("HTTP error: {status} - {message}")] HttpError {
        status: u16,
        message: String,
    },
}

impl From<SpecGenError> for AxiomError {
    fn from(err: SpecGenError) -> Self {
        match err {
            SpecGenError::ApiError(msg) =>
                AxiomError::ExternalToolError {
                    tool: "LLM API".to_string(),
                    message: msg,
                },
            SpecGenError::ConfigError(err) => AxiomError::SystemError(err.to_string()),
            SpecGenError::ParseError(msg) => AxiomError::SpecTranslationError(msg),
            SpecGenError::TemplateError(msg) => AxiomError::SpecTranslationError(msg),
            SpecGenError::ValidationError(msg) => AxiomError::SpecificationError(msg),
            SpecGenError::NetworkError(msg) =>
                AxiomError::ExternalToolError {
                    tool: "Network".to_string(),
                    message: msg,
                },
            SpecGenError::IoError(err) => AxiomError::SystemError(err.to_string()),
            SpecGenError::SerdeError(err) => AxiomError::SpecTranslationError(err.to_string()),
            SpecGenError::HttpError { status, message } =>
                AxiomError::ExternalToolError {
                    tool: "HTTP".to_string(),
                    message: format!("Status {}: {}", status, message),
                },
        }
    }
}

/// Domain-specific context information
#[derive(Clone)]
struct DomainContext {
    description: String,
    common_properties: Vec<String>,
    example_snippets: Vec<String>,
    verification_advice: String,
}

/// OpenAI API request and response types
#[derive(Debug, Serialize, Deserialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ChatResponseChoice {
    message: ChatMessage,
    finish_reason: String,
    index: usize,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<ChatResponseChoice>,
    usage: Usage,
}

/// Implementation of the SpecificationGenerator trait
/// LLMSpecificationGenerator uses LLMs to generate and translate formal specifications
///
/// # API Keys
/// This implementation requires an API key for accessing LLM services.
/// You can provide this key in one of two ways:
///
/// 1. Set in the configuration via the `api_key` field
/// 2. Set an environment variable based on the LLM provider:
///    - OpenAI: OPENAI_API_KEY
///    - Anthropic: ANTHROPIC_API_KEY
///    - Azure OpenAI: AZURE_OPENAI_API_KEY
///    - Mistral: MISTRAL_API_KEY
///    - Together: TOGETHER_API_KEY
///
/// # Usage Example
/// ```rust,no_run
/// use axiom::{
///     LLMSpecificationGenerator,
///     models::common::{Domain, VerificationLanguage},
///     models::specification::SpecificationOptions,
///     traits::specification_generator::SpecificationGenerator,
/// };
///
/// async fn generate_example() -> Result<(), Box<dyn std::error::Error>> {
///     // Create the generator
///     let generator = LLMSpecificationGenerator::new_with_defaults();
///
///     // Define requirements
///     let requirements = vec![
///         "The system must encrypt data using AES-256".to_string(),
///         "Keys must be rotated every 30 days".to_string(),
///     ];
///
///     // Configure options
///     let mut options = SpecificationOptions::default();
///     options.verification_language = VerificationLanguage::FStarLang;
///
///     // Generate the specification
///     let spec = generator.generate_specification(
///         &requirements,
///         Domain::Cryptography,
///         &options
///     ).await?;
///
///     println!("Generated specification:\n{}", spec.formal_spec.spec_code);
///
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct LLMSpecificationGenerator {
    config: GeneratorConfig,
    http_client: reqwest::Client,
    domain_contexts: HashMap<String, DomainContext>,
}

impl LLMSpecificationGenerator {
    /// Create a new LLMSpecificationGenerator with the given configuration
    pub fn new(config: GeneratorConfig) -> Self {
        let http_client = reqwest::Client
            ::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");

        let mut generator = Self {
            config,
            http_client,
            domain_contexts: HashMap::new(),
        };

        // Initialize domain contexts
        generator.init_domain_contexts();

        // Ensure projects directory exists
        if let Err(e) = std::fs::create_dir_all("projects") {
            warn!("Could not create projects directory: {}", e);
        } else {
            info!("Projects directory created or already exists");
        }

        generator
    }

    /// Get F* specific guidelines to improve code generation
    fn get_fstar_guidelines(&self) -> String {
        r#"
## F* Syntax Guidelines

1. **Module Structure**:
   - Always begin with a module declaration: `module ModuleName`
   - Use `open` statements for imports: `open FStar.All`

2. **Type Definitions**:
   - Use `type` keyword for type definitions
   - For refined types, use the syntax: `type t = x:int{x > 0}`
   - Always close type refinements with a closing brace `}`

3. **Function Declarations**:
   - Use `val` for function signatures/declarations
   - Use `let` for function implementations/definitions
   - Example: `val func: int -> int` and `let func x = x + 1`

4. **Predicates and Properties**:
   - Define predicates using `let` (not just the name)
   - Example: `let lemma_name (x: int) : Lemma (x + 0 = x) = ()`

5. **Common Errors to Avoid**:
   - Missing `let` keyword in function definitions
   - Incomplete type refinements (missing `}`)
   - Incorrect function type signatures
   - Using undefined functions or types
   
6. **Security Properties**:
   - Use `Lemma` type for security properties
   - Always include pre-conditions with `requires` and post-conditions with `ensures`

7. **Memory Management**:
   - Use the ST effect when dealing with stateful computation
   - Reference memory with `ref` type

8. **Error Handling**:
   - Use option types for operations that might fail
   - Pattern: `val safe_div: x:int -> y:int{y <> 0} -> int`
   
9. **Self-Verification**:
   - Review the specification for syntax correctness
   - Ensure all types are properly defined before use
   - Check that all functions have correct `let` definitions
"#.to_string()
    }

    // F* is the only supported verification language now

    // F* is the only supported verification language now

    /// Initialize with default configuration
    pub fn new_with_defaults() -> Self {
        Self::new(GeneratorConfig::default())
    }

    /// Simplified API for generating a specification and saving it to a project
    /// This is the main method you should use for generating specifications
    pub async fn generate_and_save(
        &self,
        project_name: &str,
        requirements: &[String],
        domain: Domain,
        language: VerificationLanguage
    ) -> AxiomResult<(Specification, PathBuf)> {
        info!("Generating specification for project: {}", project_name);

        // Set up options with the specified verification language
        let mut options = SpecificationOptions::default();
        options.verification_language = language;

        // Generate the specification
        let spec = self.generate_specification(requirements, domain, &options).await?;

        // Save it to the project
        let project_dir = self.save_to_project(project_name, &spec)?;

        info!("Specification generated and saved to project: {}", project_name);
        Ok((spec, project_dir))
    }

    /// Save specification to a project folder
    pub fn save_to_project(
        &self,
        project_name: &str,
        spec: &Specification
    ) -> AxiomResult<PathBuf> {
        use std::fs::{ self, File };
        use std::io::Write;

        // Create project directory
        let project_dir = PathBuf::from("projects").join(project_name);
        fs
            ::create_dir_all(&project_dir)
            .map_err(|e|
                AxiomError::SystemError(format!("Failed to create project directory: {}", e))
            )?;

        info!("Saving specification to project: {}", project_name);

        // Save specification code
        let spec_file_ext = match spec.formal_spec.verification_language {
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

        let spec_file_path = project_dir.join(format!("specification.{}", spec_file_ext));
        let mut spec_file = File::create(&spec_file_path).map_err(|e|
            AxiomError::SystemError(format!("Failed to create specification file: {}", e))
        )?;

        spec_file
            .write_all(spec.formal_spec.spec_code.as_bytes())
            .map_err(|e|
                AxiomError::SystemError(format!("Failed to write specification file: {}", e))
            )?;

        // Save metadata
        let metadata_file_path = project_dir.join("metadata.json");
        let metadata =
            serde_json::json!({
            "id": spec.id,
            "domain": format!("{:?}", spec.metadata.domain),
            "verification_language": format!("{:?}", spec.formal_spec.verification_language),
            "verification_system": format!("{:?}", spec.metadata.verification_system),
            "created_at": spec.metadata.created_at.to_rfc3339(),
            "confidence_score": spec.metadata.confidence_score,
            "is_formally_validated": spec.metadata.is_formally_validated,
        });

        let metadata_str = serde_json
            ::to_string_pretty(&metadata)
            .map_err(|e| AxiomError::SystemError(format!("Failed to serialize metadata: {}", e)))?;

        let mut metadata_file = File::create(&metadata_file_path).map_err(|e|
            AxiomError::SystemError(format!("Failed to create metadata file: {}", e))
        )?;

        metadata_file
            .write_all(metadata_str.as_bytes())
            .map_err(|e| AxiomError::SystemError(format!("Failed to write metadata file: {}", e)))?;

        // Save requirements
        let requirements_file_path = project_dir.join("requirements.txt");
        let requirements_str = spec.source_requirements.join("\n");
        let mut requirements_file = File::create(&requirements_file_path).map_err(|e|
            AxiomError::SystemError(format!("Failed to create requirements file: {}", e))
        )?;

        requirements_file
            .write_all(requirements_str.as_bytes())
            .map_err(|e|
                AxiomError::SystemError(format!("Failed to write requirements file: {}", e))
            )?;

        info!("Specification saved to {}", spec_file_path.display());
        Ok(project_dir)
    }

    /// Initialize domain-specific context information
    fn init_domain_contexts(&mut self) {
        // Add domain context for cryptography
        self.domain_contexts.insert("cryptography".to_string(), DomainContext {
            description: "Cryptographic systems require formal verification to ensure security properties like confidentiality, integrity, and authenticity.".to_string(),
            common_properties: vec![
                "Confidentiality: Encrypted data cannot be read by unauthorized parties".to_string(),
                "Integrity: Data cannot be modified without detection".to_string(),
                "Authentication: The identity of parties can be verified".to_string(),
                "Non-repudiation: Actions cannot be denied by the party that performed them".to_string(),
                "Forward secrecy: Compromise of long-term keys does not compromise past session keys".to_string()
            ],
            example_snippets: vec![
                "lemma confidentiality (m:message, k:key, c:ciphertext):\n  requires enc(m, k) = c\n  ensures forall k'. k' != k ==> dec(c, k') != m".to_string()
            ],
            verification_advice: "Focus on proving security properties against active adversaries with defined capabilities. Consider side-channel attacks and timing vulnerabilities.".to_string(),
        });

        // Add domain context for distributed systems
        self.domain_contexts.insert("distributedsystems".to_string(), DomainContext {
            description: "Distributed systems require formal verification to ensure consistency, fault tolerance, and liveness properties across multiple nodes.".to_string(),
            common_properties: vec![
                "Safety: Bad things never happen".to_string(),
                "Liveness: Good things eventually happen".to_string(),
                "Fault tolerance: The system can recover from specified types of failures".to_string(),
                "Consistency: All nodes eventually agree on the state".to_string(),
                "Deadlock freedom: The system never reaches a state where progress is impossible".to_string()
            ],
            example_snippets: vec![
                "theorem consensus_safety:\n  forall n1, n2: Node, v1, v2: Value.\n  decided(n1, v1) && decided(n2, v2) => v1 = v2".to_string()
            ],
            verification_advice: "Use temporal logic to reason about system behavior over time. Consider all possible interleavings of events across nodes.".to_string(),
        });

        // Additional domains could be added similarly
    }

    /// Get domain-specific context as a formatted string
    fn get_domain_context(&self, domain: &Domain) -> String {
        let domain_name = match domain {
            Domain::Cryptography => "cryptography",
            Domain::DistributedSystems => "distributedsystems",
            Domain::WebSecurity => "websecurity",
            Domain::MachineLearning => "machinelearning",
            Domain::SystemsSoftware => "systemssoftware",
            Domain::Blockchain => "blockchain",
            Domain::SafetyControl => "safetycontrol",
            Domain::HighAssuranceSoftware => "highassurance",
            Domain::Custom(name) => name,
        };

        if let Some(context) = self.domain_contexts.get(domain_name) {
            format!(
                "Domain: {}\n\
                Description: {}\n\n\
                Common properties for this domain:\n{}\n\n\
                Verification advice: {}\n",
                domain_name,
                context.description,
                context.common_properties
                    .iter()
                    .map(|p| format!("- {}", p))
                    .collect::<Vec<_>>()
                    .join("\n"),
                context.verification_advice
            )
        } else {
            format!("Domain: {}\nNo specific guidance available for this domain.", domain_name)
        }
    }

    /// Render a template with the given parameters
    fn render_template(
        &self,
        template_name: &str,
        params: &HashMap<String, String>
    ) -> Result<String, SpecGenError> {
        let template = self.config
            .get_template(template_name)
            .ok_or_else(||
                SpecGenError::TemplateError(format!("Template not found: {}", template_name))
            )?;

        let mut result = template.clone();
        for (key, value) in params {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        Ok(result)
    }

    /// Call the LLM API with the given prompt
    async fn call_llm_api(&self, prompt: &str) -> Result<String, SpecGenError> {
        use log::{ debug, info, warn };

        let preferred_provider = "anthropic"; // Try Anthropic first, then fall back to other providers
        let (provider, api_key) = match self.config.get_api_key(preferred_provider) {
            Ok(result) => result,
            Err(e) => {
                warn!("API key error: {}", e);
                return Err(SpecGenError::ApiError(format!("API key error: {}", e)));
            }
        };

        // Adjust endpoint and model based on the provider
        let (api_endpoint, model) = match provider.as_str() {
            "openai" => {
                info!("Using OpenAI provider");
                (
                    self.config.llm_api.api_endpoint
                        .clone()
                        .unwrap_or_else(||
                            "https://api.openai.com/v1/chat/completions".to_string()
                        ),
                    self.config.llm_api.model.clone().unwrap_or_else(|| "gpt-4o".to_string()),
                )
            }
            "anthropic" => {
                info!("Using Anthropic provider");
                (
                    "https://api.anthropic.com/v1/messages".to_string(),
                    "claude-3-sonnet-20240229".to_string(), // Using Claude 3.7 Sonnet
                )
            }
            "azure" => {
                info!("Using Azure OpenAI provider");
                (
                    self.config.llm_api.api_endpoint
                        .clone()
                        .unwrap_or_else(|| panic!("Azure OpenAI endpoint must be configured")),
                    self.config.llm_api.model.clone().unwrap_or_else(|| "gpt-4".to_string()),
                )
            }
            "mistral" => {
                info!("Using Mistral provider");
                (
                    "https://api.mistral.ai/v1/chat/completions".to_string(),
                    "mistral-large-latest".to_string(),
                )
            }
            "together" => {
                info!("Using Together provider");
                (
                    "https://api.together.xyz/v1/completions".to_string(),
                    "llama-3-70b-instruct".to_string(),
                )
            }
            _ => {
                warn!("Unknown provider: {}, falling back to OpenAI", provider);
                ("https://api.openai.com/v1/chat/completions".to_string(), "gpt-4o".to_string())
            }
        };

        let temperature = self.config.temperature.unwrap_or(0.2);
        let max_tokens = self.config.max_tokens.unwrap_or(4096);

        // General log info about the request
        info!("Making LLM API request to {}", provider);
        debug!("API endpoint: {}", api_endpoint);
        debug!("Model: {}", model);
        debug!("Temperature: {}", temperature);
        debug!("Max tokens: {}", max_tokens);
        debug!("Prompt length: {} characters", prompt.len());

        // Simplified version - a more complete implementation would handle different API formats
        if provider == "anthropic" {
            // For Anthropic (Claude API format)
            info!("Using Anthropic (Claude) API format");

            // Anthropic-specific request format
            let claude_request =
                serde_json::json!({
                "model": model,
                "max_tokens": max_tokens,
                "temperature": temperature,
                "system": "You are a formal verification expert who creates precise, detailed formal specifications.",
                "messages": [
                    {
                        "role": "user",
                        "content": prompt
                    }
                ]
            });

            debug!(
                "Claude request: {}",
                serde_json::to_string(&claude_request).unwrap_or_default()
            );

            debug!("Sending request to Anthropic API");
            debug!("Anthropic API endpoint: {}", api_endpoint);

            let request_builder = self.http_client
                .post(&api_endpoint)
                .header("Content-Type", "application/json")
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01") // Use the current API version for Claude
                .json(&claude_request);

            let response = request_builder.send().await.map_err(|e| {
                let error_msg = format!("Network error when calling Anthropic API: {}", e);
                warn!("{}", error_msg);
                if e.is_timeout() {
                    warn!("Request timed out");
                }
                if e.is_connect() {
                    warn!("Connection error - check network connectivity");
                }
                if e.is_request() {
                    warn!("Request construction error");
                }
                SpecGenError::NetworkError(error_msg)
            })?;

            if !response.status().is_success() {
                let status = response.status().as_u16();
                let error_text = response
                    .text().await
                    .unwrap_or_else(|_| "Failed to get error message".to_string());

                warn!("API error: HTTP {} - {}", status, error_text);
                return Err(SpecGenError::HttpError {
                    status,
                    message: error_text,
                });
            }

            // Parse Anthropic response format
            let response_text = response.text().await.map_err(|e| {
                warn!("Failed to get response text: {}", e);
                SpecGenError::ParseError(e.to_string())
            })?;

            info!("Successfully received response from Anthropic API");
            debug!("Response length: {} characters", response_text.len());

            // Parse the response to extract content
            let response_json: serde_json::Value = serde_json
                ::from_str(&response_text)
                .map_err(|e| {
                    warn!("JSON parsing error: {}", e);
                    SpecGenError::ParseError(e.to_string())
                })?;

            debug!(
                "Anthropic response structure: {}",
                serde_json
                    ::to_string_pretty(&response_json)
                    .unwrap_or_else(|_| "unable to format".to_string())
            );

            // Extract content based on the Anthropic Claude API response structure
            let content = if let Some(content_array) = response_json["content"].as_array() {
                if let Some(first_content) = content_array.get(0) {
                    if let Some(text) = first_content["text"].as_str() {
                        text.to_string()
                    } else {
                        warn!("Failed to extract text from Anthropic response content");
                        return Err(
                            SpecGenError::ParseError(
                                "Missing text in Anthropic response content".to_string()
                            )
                        );
                    }
                } else {
                    warn!("No content items in Anthropic response");
                    return Err(
                        SpecGenError::ParseError(
                            "Empty content array in Anthropic response".to_string()
                        )
                    );
                }
            } else {
                // Try alternative response structure (in case API changed)
                response_json["content"]
                    .as_str()
                    .or_else(|| response_json["completion"].as_str())
                    .ok_or_else(|| {
                        warn!("Failed to extract content from Anthropic response");
                        SpecGenError::ParseError(
                            "Unable to find content in Anthropic response".to_string()
                        )
                    })?
                    .to_string()
            };

            info!("Successfully extracted content from Anthropic response");
            debug!("Content length: {} characters", content.len());

            Ok(content)
        } else {
            // For OpenAI and similar APIs
            info!("Using OpenAI-compatible API format");

            let request = ChatRequest {
                model: model.clone(),
                messages: vec![
                    ChatMessage {
                        role: "system".to_string(),
                        content: "You are a formal verification expert who creates precise, detailed formal specifications.".to_string(),
                    },
                    ChatMessage {
                        role: "user".to_string(),
                        content: prompt.to_string(),
                    }
                ],
                temperature,
                max_tokens,
                stream: None,
            };

            debug!("Calling LLM API with prompt: {} (truncated)", if prompt.len() > 100 {
                &prompt[0..100]
            } else {
                prompt
            });

            // Different auth header for different providers
            let auth_header = match provider.as_str() {
                "azure" => format!("Bearer {}", api_key),
                "mistral" => format!("Bearer {}", api_key),
                "together" => format!("Bearer {}", api_key),
                _ => format!("Bearer {}", api_key), // Default for OpenAI
            };

            debug!("Sending request to API endpoint");
            debug!("OpenAI API endpoint: {}", api_endpoint);

            let request_builder = self.http_client
                .post(&api_endpoint)
                .header("Content-Type", "application/json")
                .header("Authorization", auth_header)
                .json(&request);

            let response = request_builder.send().await.map_err(|e| {
                let error_msg = format!("Network error when calling OpenAI API: {}", e);
                warn!("{}", error_msg);
                if e.is_timeout() {
                    warn!("Request timed out");
                }
                if e.is_connect() {
                    warn!("Connection error - check network connectivity");
                }
                if e.is_request() {
                    warn!("Request construction error");
                }
                SpecGenError::NetworkError(error_msg)
            })?;

            if !response.status().is_success() {
                let status = response.status().as_u16();
                let error_text = response
                    .text().await
                    .unwrap_or_else(|_| "Failed to get error message".to_string());

                warn!("API error: HTTP {} - {}", status, error_text);
                return Err(SpecGenError::HttpError {
                    status,
                    message: error_text,
                });
            }

            info!("Successfully received response from API");

            // First try to parse as a raw JSON value to inspect and debug
            let response_text = response.text().await.map_err(|e| {
                warn!("Failed to get text from response: {}", e);
                SpecGenError::ParseError(e.to_string())
            })?;

            debug!("OpenAI response length: {} characters", response_text.len());

            // Parse as JSON to inspect the structure
            let response_json_value: serde_json::Value = match serde_json::from_str(&response_text) {
                Ok(v) => v,
                Err(e) => {
                    warn!("Failed to parse response as JSON: {}", e);
                    return Err(SpecGenError::ParseError(format!("Invalid JSON response: {}", e)));
                }
            };

            debug!(
                "OpenAI response structure: {}",
                serde_json
                    ::to_string_pretty(&response_json_value)
                    .unwrap_or_else(|_| "unable to format".to_string())
            );

            // Try to extract content directly from JSON structure (more robust approach)
            if let Some(choices) = response_json_value["choices"].as_array() {
                if !choices.is_empty() {
                    if let Some(message) = choices[0]["message"].as_object() {
                        if let Some(content) = message.get("content") {
                            if let Some(text) = content.as_str() {
                                debug!("Successfully extracted content from JSON structure");
                                debug!("Response content length: {} characters", text.len());
                                info!("API call completed successfully");
                                return Ok(text.to_string());
                            }
                        }
                    }
                }
            }

            // Fallback: Try to parse using the struct
            warn!("Direct extraction failed, trying to parse using the struct");
            let response_json: ChatResponse = match serde_json::from_str(&response_text) {
                Ok(r) => r,
                Err(e) => {
                    warn!("Failed to parse response using structured approach: {}", e);
                    return Err(
                        SpecGenError::ParseError(format!("Failed to parse API response: {}", e))
                    );
                }
            };

            if response_json.choices.is_empty() {
                warn!("API returned empty choices array");
                return Err(SpecGenError::ApiError("No response from API".to_string()));
            }

            let content = response_json.choices[0].message.content.clone();
            debug!("Response content length: {} characters", content.len());
            info!("API call completed successfully");

            Ok(content)
        }
    }

    /// Create a FormalSpecification from the LLM response
    fn parse_formal_specification(
        &self,
        content: &str,
        verification_language: VerificationLanguage
    ) -> Result<FormalSpecification, SpecGenError> {
        let mut components = HashMap::new();

        // Extract just the code from code blocks
        let mut in_code_block = false;
        let mut extracted_code = String::new();
        let mut current_component = String::new();
        let mut current_component_name = String::new();

        info!("Extracting formal specification code from LLM response");

        // Store the full response as documentation
        components.insert("description".to_string(), content.to_string());

        let lines: Vec<&str> = content.lines().collect();
        for line in lines.iter() {
            if line.starts_with("```") {
                in_code_block = !in_code_block;

                // When exiting a code block, save the component
                if !in_code_block && !current_component.is_empty() {
                    components.insert(current_component_name.clone(), current_component.clone());
                    current_component.clear();
                }

                // Determine language/component name if specified
                if line.len() > 3 && in_code_block {
                    current_component_name = format!("component_{}", components.len() + 1);
                }

                continue;
            }

            // Only process lines within code blocks
            if in_code_block {
                // Add to the current component
                if !current_component_name.is_empty() {
                    current_component.push_str(line);
                    current_component.push('\n');
                }

                // Also add to the main extracted code
                extracted_code.push_str(line);
                extracted_code.push('\n');
            }
        }

        // Save any remaining component
        if !current_component.is_empty() && !current_component_name.is_empty() {
            components.insert(current_component_name, current_component);
        }

        // If no code blocks found, use the whole response
        if extracted_code.is_empty() {
            info!("No code blocks found in response, using entire response");
            extracted_code = content.to_string();
        } else {
            info!("Successfully extracted code blocks from response");
        }

        // Extract dependencies (imports or includes mentioned in the code)
        let dependencies = extract_dependencies(&extracted_code, &verification_language);

        // Create the formal specification with just the extracted code
        let spec = FormalSpecification {
            verification_language,
            spec_code: extracted_code,
            components,
            dependencies,
        };

        info!("Extracted specification code of {} characters", spec.spec_code.len());
        Ok(spec)
    }
}

/// Helper function to extract dependencies from code
fn extract_dependencies(content: &str, language: &VerificationLanguage) -> Vec<String> {
    let mut dependencies = Vec::new();

    // Different languages have different import/include syntax
    let patterns = match language {
        VerificationLanguage::FStarLang => vec!["open ", "include ", "module "],
        VerificationLanguage::DafnyLang => vec!["import "],
        VerificationLanguage::CoqLang => vec!["Require Import ", "Require Export "],
        VerificationLanguage::IsabelleLang => vec!["imports "],
        VerificationLanguage::LeanLang => vec!["import "],
        VerificationLanguage::TLAPlus => vec!["EXTENDS "],
        VerificationLanguage::Why3Lang => vec!["use "],
        VerificationLanguage::Z3SMT => vec!["(include "],
        _ => vec![],
    };

    if patterns.is_empty() {
        return dependencies;
    }

    for line in content.lines() {
        let trimmed = line.trim();
        for pattern in &patterns {
            if trimmed.starts_with(pattern) {
                let dep = trimmed[pattern.len()..]
                    .trim()
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_end_matches(|c| (c == ',' || c == ';'));

                if !dep.is_empty() {
                    dependencies.push(dep.to_string());
                }
            }
        }
    }

    dependencies
}

// Implement the SpecificationGenerator trait for LLMSpecificationGenerator
#[async_trait]
impl SpecificationGenerator for LLMSpecificationGenerator {
    async fn generate_specification(
        &self,
        requirements: &[String],
        domain: Domain,
        options: &SpecificationOptions
    ) -> AxiomResult<Specification> {
        info!("Generating specification for domain: {:?}", domain);

        // Get the appropriate template
        let template_name = "specification";

        // Get language-specific guidelines for improved code generation
        let language_guidelines = match options.verification_language {
            VerificationLanguage::FStarLang => self.get_fstar_guidelines(),
            VerificationLanguage::DafnyLang => self.get_dafny_guidelines(),
            VerificationLanguage::CoqLang => self.get_coq_guidelines(),
            _ => String::new(),
        };

        // Prepare template parameters
        let mut params = HashMap::new();
        params.insert("domain".to_string(), domain.to_string());
        params.insert(
            "verification_language".to_string(),
            options.verification_language.to_string()
        );
        params.insert("requirements".to_string(), requirements.join("\n"));
        params.insert("domain_context".to_string(), self.get_domain_context(&domain));
        params.insert("language_guidelines".to_string(), language_guidelines);

        // Render the template
        let prompt = self.render_template(template_name, &params).map_err(AxiomError::from)?;

        // Call the LLM API
        let response = self.call_llm_api(&prompt).await.map_err(AxiomError::from)?;

        // Parse the response into a formal specification
        let formal_spec = self
            .parse_formal_specification(&response, options.verification_language.clone())
            .map_err(AxiomError::from)?;

        // Create the specification object
        let spec = Specification {
            id: format!("spec_{}", chrono::Utc::now().timestamp()),
            source_requirements: requirements.to_vec(),
            formal_properties: vec![], // In a real implementation, we would extract these from the response
            formal_spec,
            metadata: crate::models::specification::SpecificationMetadata {
                created_at: chrono::Utc::now(),
                verification_system: match options.verification_language {
                    VerificationLanguage::FStarLang =>
                        crate::models::common::VerificationSystem::FStar,
                    VerificationLanguage::DafnyLang =>
                        crate::models::common::VerificationSystem::Dafny,
                    VerificationLanguage::CoqLang => crate::models::common::VerificationSystem::Coq,
                    VerificationLanguage::IsabelleLang =>
                        crate::models::common::VerificationSystem::Isabelle,
                    VerificationLanguage::LeanLang =>
                        crate::models::common::VerificationSystem::Lean,
                    VerificationLanguage::TLAPlus => crate::models::common::VerificationSystem::TLA,
                    VerificationLanguage::Why3Lang =>
                        crate::models::common::VerificationSystem::Why3,
                    VerificationLanguage::Z3SMT => crate::models::common::VerificationSystem::Z3,
                    _ =>
                        crate::models::common::VerificationSystem::Custom(
                            options.verification_language.to_string()
                        ),
                },
                domain: domain.clone(),
                confidence_score: 0.9, // In a real implementation, this would be calculated
                is_formally_validated: false,
            },
        };

        Ok(spec)
    }

    async fn refine_specification(
        &self,
        spec: &Specification,
        feedback: &str,
        _options: &SpecificationOptions
    ) -> AxiomResult<Specification> {
        info!("Refining specification based on feedback");

        // Prepare the refinement prompt
        let mut params = HashMap::new();
        params.insert(
            "verification_language".to_string(),
            spec.formal_spec.verification_language.to_string()
        );
        params.insert("original_spec".to_string(), spec.formal_spec.spec_code.clone());
        params.insert("feedback".to_string(), feedback.to_string());

        let prompt = format!(
            "You are a formal verification expert. You need to refine a formal specification based on feedback.\n\n\
            Original specification in {}:\n\
            ```\n{}\n```\n\n\
            Feedback to address:\n{}\n\n\
            Please provide a revised specification that addresses the feedback while maintaining \
            all the original requirements. Include all necessary types, functions, and properties.",
            spec.formal_spec.verification_language.to_string(),
            spec.formal_spec.spec_code,
            feedback
        );

        // Call the LLM API
        let response = self.call_llm_api(&prompt).await.map_err(AxiomError::from)?;

        // Parse the response into a formal specification
        let formal_spec = self
            .parse_formal_specification(&response, spec.formal_spec.verification_language.clone())
            .map_err(AxiomError::from)?;

        // Create a new specification with the refined formal spec
        let refined_spec = Specification {
            id: format!("{}_refined", spec.id),
            source_requirements: spec.source_requirements.clone(),
            formal_properties: spec.formal_properties.clone(), // In a real implementation, these might be updated
            formal_spec,
            metadata: crate::models::specification::SpecificationMetadata {
                created_at: chrono::Utc::now(),
                verification_system: spec.metadata.verification_system.clone(),
                domain: spec.metadata.domain.clone(),
                confidence_score: spec.metadata.confidence_score,
                is_formally_validated: false,
            },
        };

        Ok(refined_spec)
    }

    async fn validate_specification(
        &self,
        spec: &Specification,
        validation_depth: ValidationDepth
    ) -> AxiomResult<ValidationReport> {
        info!("Validating specification with depth: {}", match validation_depth {
            ValidationDepth::Basic => "Basic",
            ValidationDepth::TypeCheck => "Type Check",
            ValidationDepth::FormalVerification => "Formal Verification",
        });

        // Perform validation based on the requested depth
        let validation_report = match validation_depth {
            ValidationDepth::Basic => self.validate_syntax(spec).await?,
            ValidationDepth::TypeCheck => self.validate_type_checking(spec).await?,
            ValidationDepth::FormalVerification => self.validate_formal_verification(spec).await?,
        };

        // If the validation failed, attempt to fix the issues automatically
        if !validation_report.is_valid {
            info!(
                "Validation failed with {} issues. Attempting to fix automatically...",
                validation_report.issues.len()
            );

            // Try to fix the specification
            match self.fix_specification_with_retry(spec, &validation_report).await {
                Ok(fixed_report) => {
                    info!("Auto-fixed specification validation report: {}", if
                        fixed_report.is_valid
                    {
                        "valid"
                    } else {
                        "still invalid"
                    });
                    return Ok(fixed_report);
                }
                Err(e) => {
                    warn!("Failed to automatically fix specification: {}", e);
                    // Return the original validation report if fixing failed
                    return Ok(validation_report);
                }
            }
        }

        Ok(validation_report)
    }

    async fn translate_to_properties(
        &self,
        requirements: &[String],
        domain: Domain
    ) -> AxiomResult<Vec<SpecificationTranslation>> {
        info!("Translating requirements to properties for domain: {}", domain);

        // Prepare the prompt for property extraction
        let prompt = format!(
            "You are a formal verification expert. Extract formal properties from these requirements for a {} system:\n\n{}\n\n\
            For each requirement, provide:\n\
            1. The formal interpretation as a property\n\
            2. The property expressed in a mathematical notation\n\
            3. A confidence score (0-1) for your translation\n\
            Format each property as: \"Requirement: [original text]\\nFormal property: [interpretation]\\nMathematical form: [formal notation]\\nConfidence: [score]\"",
            domain.to_string(),
            requirements.join("\n")
        );

        // Call the LLM API
        let response = self.call_llm_api(&prompt).await.map_err(AxiomError::from)?;

        // Parse the response into property translations
        let mut translations = Vec::new();

        // Simple parser for the response format
        let sections = response.split("\n\n").collect::<Vec<_>>();

        for section in sections {
            if section.is_empty() {
                continue;
            }

            let lines: Vec<&str> = section.lines().collect();
            if lines.len() < 4 {
                continue;
            }

            let requirement = lines[0]
                .strip_prefix("Requirement: ")
                .unwrap_or(lines[0])
                .to_string();

            let interpreted_property = lines[1]
                .strip_prefix("Formal property: ")
                .unwrap_or(lines[1])
                .to_string();

            let formal_representation = lines[2]
                .strip_prefix("Mathematical form: ")
                .unwrap_or(lines[2])
                .to_string();

            let confidence_str = lines[3].strip_prefix("Confidence: ").unwrap_or("0.7").trim();

            let translation_confidence = confidence_str.parse::<f32>().unwrap_or(0.7);

            // Determine if human review is needed based on confidence
            let requires_human_review = translation_confidence < 0.8;

            translations.push(SpecificationTranslation {
                requirement,
                interpreted_properties: vec![interpreted_property],
                formal_representation,
                translation_confidence,
                verification_language: VerificationLanguage::FStarLang, // Default - would be adjustable
                requires_human_review,
            });
        }

        Ok(translations)
    }

    async fn convert_to_formal_specification(
        &self,
        translations: &[SpecificationTranslation],
        target_language: VerificationLanguage,
        paradigm: SpecificationParadigm
    ) -> AxiomResult<FormalSpecification> {
        info!("Converting properties to formal specification in {}", target_language);

        // Prepare a prompt with all the properties
        let properties_text = translations
            .iter()
            .map(|t|
                format!(
                    "Requirement: {}\nFormal Property: {}\nFormalization: {}",
                    t.requirement,
                    t.interpreted_properties.join(", "),
                    t.formal_representation
                )
            )
            .collect::<Vec<_>>()
            .join("\n\n");

        let paradigm_str = match paradigm {
            SpecificationParadigm::PrePostConditions => "pre and post conditions",
            SpecificationParadigm::TypeTheoretic => "type theory",
            SpecificationParadigm::ModelChecking => "model checking",
            SpecificationParadigm::TemporalLogic => "temporal logic",
            SpecificationParadigm::Refinement => "refinement types",
            SpecificationParadigm::HoareLogic => "Hoare logic",
            SpecificationParadigm::SeparationLogic => "separation logic",
            SpecificationParadigm::Custom(ref s) => s,
        };

        let prompt = format!(
            "You are a formal verification expert. Convert these formal properties into a complete {} specification using {}:\n\n\
            {}\n\n\
            Generate a complete, well-structured formal specification that captures all these properties. \
            Include all necessary type definitions, functions, and verification statements. \
            Format your response as a valid {} specification that could be directly input to the verification tool.",
            target_language.to_string(),
            paradigm_str,
            properties_text,
            target_language.to_string()
        );

        // Call the LLM API
        let response = self.call_llm_api(&prompt).await.map_err(AxiomError::from)?;

        // Parse the response into a formal specification
        let formal_spec = self
            .parse_formal_specification(&response, target_language)
            .map_err(AxiomError::from)?;

        Ok(formal_spec)
    }

    async fn translate_specification(
        &self,
        spec: &Specification,
        target_language: VerificationLanguage
    ) -> AxiomResult<Specification> {
        info!(
            "Translating specification from {} to {}",
            spec.formal_spec.verification_language,
            target_language
        );

        if spec.formal_spec.verification_language == target_language {
            return Ok(spec.clone());
        }

        // Prepare the translation prompt
        let prompt = format!(
            "You are a formal verification expert. Translate this {} specification to {}:\n\n\
            ```\n{}\n```\n\n\
            Ensure that all properties and semantics are preserved in the translation. \
            Format your response as a valid {} specification.",
            spec.formal_spec.verification_language.to_string(),
            target_language.to_string(),
            spec.formal_spec.spec_code,
            target_language.to_string()
        );

        // Call the LLM API
        let response = self.call_llm_api(&prompt).await.map_err(AxiomError::from)?;

        // Parse the response into a formal specification
        let formal_spec = self
            .parse_formal_specification(&response, target_language.clone())
            .map_err(AxiomError::from)?;

        // Create a new specification with the translated formal spec
        let translated_spec = Specification {
            id: format!("{}_translated", spec.id),
            source_requirements: spec.source_requirements.clone(),
            formal_properties: spec.formal_properties.clone(),
            formal_spec,
            metadata: crate::models::specification::SpecificationMetadata {
                created_at: chrono::Utc::now(),
                verification_system: match target_language {
                    VerificationLanguage::FStarLang =>
                        crate::models::common::VerificationSystem::FStar,
                    VerificationLanguage::DafnyLang =>
                        crate::models::common::VerificationSystem::Dafny,
                    VerificationLanguage::CoqLang => crate::models::common::VerificationSystem::Coq,
                    VerificationLanguage::IsabelleLang =>
                        crate::models::common::VerificationSystem::Isabelle,
                    VerificationLanguage::LeanLang =>
                        crate::models::common::VerificationSystem::Lean,
                    VerificationLanguage::TLAPlus => crate::models::common::VerificationSystem::TLA,
                    VerificationLanguage::Why3Lang =>
                        crate::models::common::VerificationSystem::Why3,
                    VerificationLanguage::Z3SMT => crate::models::common::VerificationSystem::Z3,
                    _ =>
                        crate::models::common::VerificationSystem::Custom(
                            target_language.to_string()
                        ),
                },
                domain: spec.metadata.domain.clone(),
                confidence_score: spec.metadata.confidence_score * 0.9, // Slight reduction due to translation
                is_formally_validated: false,
            },
        };

        Ok(translated_spec)
    }

    async fn verify_specification_completeness(
        &self,
        spec: &Specification,
        requirements: &[String]
    ) -> AxiomResult<(bool, Vec<String>)> {
        info!("Verifying specification completeness against requirements");

        // Prepare the prompt
        let prompt = format!(
            "You are a formal verification expert. Check if this {} specification completely covers all requirements:\n\n\
            Specification:\n```\n{}\n```\n\n\
            Requirements:\n{}\n\n\
            For each requirement, indicate whether it is fully covered, partially covered, or not covered by the specification. \
            List any requirements that are not fully covered, explaining what aspects are missing. \
            Finally, provide a boolean judgment: Is the specification complete (true/false)?",
            spec.formal_spec.verification_language.to_string(),
            spec.formal_spec.spec_code,
            requirements
                .iter()
                .map(|r| format!("- {}", r))
                .collect::<Vec<_>>()
                .join("\n")
        );

        // Call the LLM API
        let response = self.call_llm_api(&prompt).await.map_err(AxiomError::from)?;

        // Parse the response to determine completeness
        let is_complete =
            response.to_lowercase().contains("the specification is complete: true") ||
            response.to_lowercase().contains("is the specification complete? true");

        // Extract missing requirements
        let mut missing_requirements = Vec::new();

        for line in response.lines() {
            if line.contains("not covered") || line.contains("partially covered") {
                if let Some(req_start) = line.find("- ") {
                    let req = line[req_start + 2..].trim().to_string();
                    missing_requirements.push(req);
                }
            }
        }

        Ok((is_complete, missing_requirements))
    }

    async fn generate_verification_code(
        &self,
        spec: &Specification,
        target_system: crate::models::common::VerificationSystem
    ) -> AxiomResult<String> {
        info!("Generating verification code for {}", target_system);

        // Check if we need to translate the specification
        let target_language = match target_system {
            crate::models::common::VerificationSystem::FStar => VerificationLanguage::FStarLang,
            crate::models::common::VerificationSystem::Dafny => VerificationLanguage::DafnyLang,
            crate::models::common::VerificationSystem::Coq => VerificationLanguage::CoqLang,
            crate::models::common::VerificationSystem::Isabelle =>
                VerificationLanguage::IsabelleLang,
            crate::models::common::VerificationSystem::Lean => VerificationLanguage::LeanLang,
            crate::models::common::VerificationSystem::TLA => VerificationLanguage::TLAPlus,
            crate::models::common::VerificationSystem::Why3 => VerificationLanguage::Why3Lang,
            crate::models::common::VerificationSystem::Z3 => VerificationLanguage::Z3SMT,
            _ => spec.formal_spec.verification_language.clone(),
        };

        let specification_code = if spec.formal_spec.verification_language == target_language {
            spec.formal_spec.spec_code.clone()
        } else {
            let translated_spec = self.translate_specification(
                spec,
                target_language.clone()
            ).await?;
            translated_spec.formal_spec.spec_code
        };

        // For some verification systems, we might need to add additional verification code
        // Here we'll simulate that with an LLM prompt
        let prompt = format!(
            "You are a formal verification expert. Generate executable verification code for this {} specification \
            that can be used with {}:\n\n\
            ```\n{}\n```\n\n\
            Add any necessary verification directives, proof scripts, or commands needed to verify this specification. \
            The result should be a complete file that can be directly verified using the appropriate tool.",
            target_language.to_string(),
            target_system.to_string(),
            specification_code
        );

        // Call the LLM API
        let response = self.call_llm_api(&prompt).await.map_err(AxiomError::from)?;

        // Extract just the verification code (not the explanatory text)
        let mut verification_code = String::new();
        let mut in_code_block = false;

        for line in response.lines() {
            if line.starts_with("```") {
                in_code_block = !in_code_block;
                continue;
            }

            if in_code_block {
                verification_code.push_str(line);
                verification_code.push('\n');
            }
        }

        // If no code blocks found, use the whole response
        if verification_code.is_empty() {
            verification_code = response;
        }

        Ok(verification_code)
    }

    async fn get_specification_templates(
        &self,
        domain: Domain,
        language: VerificationLanguage
    ) -> AxiomResult<Vec<VerificationTemplate>> {
        info!("Getting specification templates for domain {} in language {}", domain, language);

        // In a real implementation, this would load templates from a repository or database
        // Here we're generating them on-the-fly with an LLM

        let prompt = format!(
            "You are a formal verification expert. Generate 3 template examples for {} specifications in {} for the {} domain. \
            Each template should be a complete code example that can be parameterized. \
            For each template, provide:\n\
            1. A name describing its purpose\n\
            2. The template code\n\
            3. A list of placeholders that need to be filled in\n\
            4. A brief documentation explaining how to use the template\n\
            Format each template as: \"Name: [name]\\nTemplate:\\n```\\n[code]\\n```\\nPlaceholders: [list]\\nDocumentation: [explanation]\"",
            language.to_string(),
            language.to_string(),
            domain.to_string()
        );

        // Call the LLM API
        let response = self.call_llm_api(&prompt).await.map_err(AxiomError::from)?;

        // Parse the response into templates
        let mut templates = Vec::new();

        // Split the response into template sections
        let sections = response.split("\n\nName:").collect::<Vec<_>>();

        for (i, section) in sections.iter().enumerate() {
            if i == 0 && !section.starts_with("Name:") {
                continue; // Skip introduction text
            }

            let content = if i == 0 { *section } else { &format!("Name:{}", *section) };

            // Parse template components
            let name_pattern = if i == 0 { "Name:" } else { "" };

            let name = match content.find(name_pattern) {
                Some(idx) => {
                    let start = idx + name_pattern.len();
                    let end = content[start..]
                        .find('\n')
                        .map(|e| start + e)
                        .unwrap_or(content.len());
                    content[start..end].trim().to_string()
                }
                None => format!("Template {}", i + 1),
            };

            let template_code = match content.find("```") {
                Some(start_idx) => {
                    let code_start = start_idx + 3;
                    let code_end = content[code_start..]
                        .find("```")
                        .map(|e| code_start + e)
                        .unwrap_or(content.len());
                    content[code_start..code_end].trim().to_string()
                }
                None => "// Template code not found".to_string(),
            };

            let placeholders_start = content.find("Placeholders:");
            let doc_start = content.find("Documentation:");

            let placeholders = match (placeholders_start, doc_start) {
                (Some(start), Some(end)) => {
                    let ph_text = &content[start + "Placeholders:".len()..end];
                    ph_text
                        .lines()
                        .map(|l| l.trim())
                        .filter(|l| !l.is_empty())
                        .map(|l| l.to_string())
                        .collect::<Vec<_>>()
                }
                (Some(start), None) => {
                    let ph_text = &content[start + "Placeholders:".len()..];
                    ph_text
                        .lines()
                        .map(|l| l.trim())
                        .filter(|l| !l.is_empty())
                        .map(|l| l.to_string())
                        .collect::<Vec<_>>()
                }
                _ => vec![],
            };

            let documentation = match doc_start {
                Some(start) => content[start + "Documentation:".len()..].trim().to_string(),
                None => "No documentation provided".to_string(),
            };

            templates.push(VerificationTemplate {
                language: language.clone(),
                template_name: name,
                template_code,
                placeholders,
                documentation,
            });
        }

        Ok(templates)
    }

    async fn apply_template(
        &self,
        template: &VerificationTemplate,
        properties: &[Property]
    ) -> AxiomResult<FormalSpecification> {
        info!("Applying template {} to generate specification", template.template_name);

        // Prepare the prompt
        let properties_text = properties
            .iter()
            .map(|p| format!("Property {}: {} - {}", p.id, p.description, p.formal_definition))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "You are a formal verification expert. Apply this template to generate a formal specification for these properties:\n\n\
            Template: {}\n\n\
            ```\n{}\n```\n\n\
            Properties:\n{}\n\n\
            Fill in the template placeholders using these properties. The result should be a complete {} specification. \
            Return only the filled template, no explanations.",
            template.template_name,
            template.template_code,
            properties_text,
            template.language.to_string()
        );

        // Call the LLM API
        let response = self.call_llm_api(&prompt).await.map_err(AxiomError::from)?;

        // Parse the response into a formal specification
        let formal_spec = self
            .parse_formal_specification(&response, template.language.clone())
            .map_err(AxiomError::from)?;

        Ok(formal_spec)
    }

    async fn export_specification(
        &self,
        spec: &Specification,
        output_path: &Path
    ) -> AxiomResult<()> {
        info!("Exporting specification to {}", output_path.display());

        // Ensure the directory exists
        if let Some(parent) = output_path.parent() {
            std::fs
                ::create_dir_all(parent)
                .map_err(|e|
                    AxiomError::SystemError(format!("Failed to create directory: {}", e))
                )?;
        }

        // Write the specification to the file
        std::fs
            ::write(output_path, &spec.formal_spec.spec_code)
            .map_err(|e| AxiomError::SystemError(format!("Failed to write specification: {}", e)))?;

        Ok(())
    }

    async fn import_specification(
        &self,
        spec_file: &Path,
        language: VerificationLanguage
    ) -> AxiomResult<Specification> {
        info!("Importing specification from {}", spec_file.display());

        // Read the specification file
        let spec_code = std::fs
            ::read_to_string(spec_file)
            .map_err(|e| AxiomError::SystemError(format!("Failed to read specification: {}", e)))?;

        // Ask the LLM to analyze the specification and extract requirements
        let prompt = format!(
            "You are a formal verification expert. Analyze this {} specification and extract the requirements it fulfills:\n\n\
            ```\n{}\n```\n\n\
            List each requirement that this specification addresses, in natural language form. \
            Also extract any formal properties defined in the specification.",
            language.to_string(),
            spec_code
        );

        // Call the LLM API
        let response = self.call_llm_api(&prompt).await.map_err(AxiomError::from)?;

        // Parse the response to extract requirements
        let mut requirements = Vec::new();
        let mut in_requirements = false;

        for line in response.lines() {
            if line.contains("Requirements:") || line.contains("Requirement:") {
                in_requirements = true;
                continue;
            }

            if in_requirements && line.starts_with("-") {
                let requirement = line.trim_start_matches("-").trim().to_string();
                if !requirement.is_empty() {
                    requirements.push(requirement);
                }
            }
        }

        // If no requirements found, use a generic one
        if requirements.is_empty() {
            requirements.push("Imported specification requirements".to_string());
        }

        // Create the formal specification
        let formal_spec = self
            .parse_formal_specification(&spec_code, language.clone())
            .map_err(AxiomError::from)?;

        // Create the specification object
        let spec = Specification {
            id: format!("import_{}", chrono::Utc::now().timestamp()),
            source_requirements: requirements,
            formal_properties: vec![], // In a real implementation, we would extract these
            formal_spec,
            metadata: crate::models::specification::SpecificationMetadata {
                created_at: chrono::Utc::now(),
                verification_system: match language {
                    VerificationLanguage::FStarLang =>
                        crate::models::common::VerificationSystem::FStar,
                    VerificationLanguage::DafnyLang =>
                        crate::models::common::VerificationSystem::Dafny,
                    VerificationLanguage::CoqLang => crate::models::common::VerificationSystem::Coq,
                    VerificationLanguage::IsabelleLang =>
                        crate::models::common::VerificationSystem::Isabelle,
                    VerificationLanguage::LeanLang =>
                        crate::models::common::VerificationSystem::Lean,
                    VerificationLanguage::TLAPlus => crate::models::common::VerificationSystem::TLA,
                    VerificationLanguage::Why3Lang =>
                        crate::models::common::VerificationSystem::Why3,
                    VerificationLanguage::Z3SMT => crate::models::common::VerificationSystem::Z3,
                    _ => crate::models::common::VerificationSystem::Custom(language.to_string()),
                },
                domain: Domain::Custom("imported".to_string()),
                confidence_score: 0.8,
                is_formally_validated: false,
            },
        };

        Ok(spec)
    }

    fn get_error_context(&self, _error: &str, _spec: &Specification) -> ErrorContext {
        // In a real implementation, this would analyze the error and return contextual information
        ErrorContext {
            source_location: None,
            related_requirement: None,
            stack_trace: vec![],
            suggestion: Some("Check the formal specification syntax and structure.".to_string()),
            severity: ErrorSeverity::Error,
        }
    }
}

// Implementation helpers for validation
impl LLMSpecificationGenerator {
    /// Fix specification issues identified during validation and retry validation until successful
    async fn fix_specification_with_retry(
        &self,
        spec: &Specification,
        report: &ValidationReport
    ) -> AxiomResult<ValidationReport> {
        if report.is_valid {
            return Ok(report.clone());
        }

        // Maximum number of retry attempts
        const MAX_RETRIES: usize = 3;
        let mut current_spec = spec.clone();
        let mut current_report = report.clone();

        for attempt in 1..=MAX_RETRIES {
            info!("Auto-fix attempt {} of {}", attempt, MAX_RETRIES);

            // Try to fix the current issues
            let fixed_spec = self.fix_specification(&current_spec, &current_report).await?;

            // Validate the fixed specification
            let validation_depth = if
                current_report.issues
                    .iter()
                    .any(
                        |i| matches!(i.severity, IssueSeverity::Error) && i.message.contains("type")
                    )
            {
                ValidationDepth::TypeCheck // Upgrade to type checking if there are type errors
            } else {
                ValidationDepth::Basic // Start with basic syntax validation
            };

            info!("Validating fixed specification (depth: {:?})...", validation_depth);
            let new_report = match validation_depth {
                ValidationDepth::Basic => self.validate_syntax(&fixed_spec).await?,
                ValidationDepth::TypeCheck => self.validate_type_checking(&fixed_spec).await?,
                ValidationDepth::FormalVerification =>
                    self.validate_formal_verification(&fixed_spec).await?,
            };

            if new_report.is_valid {
                // Success! Return the valid report and fixed spec
                info!("Auto-fixed specification is valid!");

                // Return a customized report that indicates automatic fixing was successful
                let mut success_report = new_report.clone();
                success_report.issues.push(ValidationIssue {
                    severity: IssueSeverity::Info,
                    message: format!("Specification was automatically fixed after {} attempts", attempt),
                    related_property: None,
                    line_number: None,
                    suggested_fix: Some(fixed_spec.formal_spec.spec_code.clone()),
                });
                return Ok(success_report);
            }

            // If still not valid, update and try again with the fixed spec
            info!("Fixed specification still has {} issues. Retrying...", new_report.issues.len());
            current_spec = fixed_spec;
            current_report = new_report;
        }

        // If we've exhausted all retries, include the best fixed code in the report
        let mut final_report = current_report.clone();
        final_report.issues.push(ValidationIssue {
            severity: IssueSeverity::Warning,
            message: format!("Automatic fixing was attempted {} times but issues remain", MAX_RETRIES),
            related_property: None,
            line_number: None,
            suggested_fix: Some(current_spec.formal_spec.spec_code.clone()),
        });

        Ok(final_report)
    }

    /// Fix specification issues identified during validation
    async fn fix_specification(
        &self,
        spec: &Specification,
        report: &ValidationReport
    ) -> AxiomResult<Specification> {
        if report.is_valid {
            return Ok(spec.clone());
        }

        // Analyze issues to determine what needs to be fixed
        let mut missing_functions = Vec::new();
        let mut syntax_issues = Vec::new();
        let mut type_errors = Vec::new();

        for issue in &report.issues {
            let msg = issue.message.to_lowercase();
            if msg.contains("undefined") && (msg.contains("function") || msg.contains("predicate")) {
                // Extract the function name
                if let Some(name) = Self::extract_name_from_error(&issue.message) {
                    missing_functions.push(name);
                }
            } else if msg.contains("syntax") || msg.contains("expected") || msg.contains("missing") {
                syntax_issues.push(issue);
            } else if msg.contains("type") {
                type_errors.push(issue);
            }
        }

        // Create a prompt with the specification and issues
        let issue_list = report.issues
            .iter()
            .map(|issue| {
                let severity = match issue.severity {
                    IssueSeverity::Error => "Error",
                    IssueSeverity::Warning => "Warning",
                    IssueSeverity::Info => "Info",
                };

                let line_info = if let Some(line) = issue.line_number {
                    format!("Line {}", line)
                } else {
                    "Unknown location".to_string()
                };

                let fix_info = if let Some(fix) = &issue.suggested_fix {
                    format!("Suggested fix: {}", fix)
                } else {
                    String::new()
                };

                format!("{}: {} - {}\n{}", line_info, issue.message, severity, fix_info)
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        // Always use F* guidelines since we only support F*
        let language_guidelines = self.get_fstar_guidelines();

        // Add specific fixing requirements based on issue analysis
        let mut specific_fixes = String::new();

        if !missing_functions.is_empty() {
            specific_fixes.push_str(
                &format!(
                    "IMPORTANT: The following functions/predicates are undefined and MUST be implemented:\n{}\n\n",
                    missing_functions
                        .iter()
                        .map(|f| format!("- `{}`", f))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            );
        }

        if !syntax_issues.is_empty() {
            specific_fixes.push_str(
                "IMPORTANT: Fix all syntax errors, ensuring proper keywords (let/val), closing braces, and proper F* syntax.\n\n"
            );
        }

        if !type_errors.is_empty() {
            specific_fixes.push_str(
                "IMPORTANT: Fix all type errors, ensuring proper typing for all expressions.\n\n"
            );
        }

        let prompt = format!(
            "You are a formal verification expert. Fix the following issues in this F* specification:\n\n\
            Original specification:\n\
            ```\n{}\n```\n\n\
            Issues to fix:\n\
            {}\n\n\
            {}\n\
            {}\n\
            Requirements:\n\
            1. Implement ALL missing functions/predicates with simple but valid implementations\n\
            2. Ensure proper syntax for F* (including all keywords and braces)\n\
            3. Fix ALL identified issues\n\
            4. Return a COMPLETE specification that preserves the original functionality\n\n\
            Return ONLY the corrected specification code without any explanations.",
            spec.formal_spec.spec_code,
            issue_list,
            specific_fixes,
            language_guidelines
        );

        // Call the LLM API to get a fixed specification
        let response = self.call_llm_api(&prompt).await.map_err(AxiomError::from)?;

        // Parse the response into a formal specification
        let fixed_formal_spec = self
            .parse_formal_specification(&response, spec.formal_spec.verification_language.clone())
            .map_err(AxiomError::from)?;

        // Create a new specification with the fixed formal spec
        let fixed_spec = Specification {
            id: format!("{}_fixed", spec.id),
            source_requirements: spec.source_requirements.clone(),
            formal_properties: spec.formal_properties.clone(),
            formal_spec: fixed_formal_spec,
            metadata: spec.metadata.clone(),
        };

        Ok(fixed_spec)
    }

    /// Helper function to extract function or predicate names from error messages
    fn extract_name_from_error(error_message: &str) -> Option<String> {
        // Common patterns in error messages for undefined functions
        let patterns = [
            "undefined function `",
            "undefined predicate `",
            "undefined identifier `",
            "unknown identifier `",
            "unbound variable `",
        ];

        for pattern in &patterns {
            if let Some(start_idx) = error_message.find(pattern) {
                let start = start_idx + pattern.len();
                if let Some(end_idx) = error_message[start..].find('`') {
                    return Some(error_message[start..start + end_idx].trim().to_string());
                }
            }
        }

        // Fallback pattern matching without backticks
        let alt_patterns = [
            "undefined function ",
            "undefined predicate ",
            "undefined identifier ",
            "unknown identifier ",
            "unbound variable ",
        ];

        for pattern in &alt_patterns {
            if let Some(start_idx) = error_message.find(pattern) {
                let start = start_idx + pattern.len();
                // Find the end of the identifier (usually followed by space, comma, or period)
                let end_idx = error_message[start..]
                    .find(|c| (c == ' ' || c == ',' || c == '.' || c == ':'))
                    .unwrap_or(error_message[start..].len());

                return Some(error_message[start..start + end_idx].trim().to_string());
            }
        }

        None
    }
    async fn validate_syntax(&self, spec: &Specification) -> AxiomResult<ValidationReport> {
        // Prepare the prompt for syntax validation
        let prompt = format!(
            "You are a formal verification expert. Validate the syntax of this {} specification:\n\n\
            ```\n{}\n```\n\n\
            Check for syntax errors, undefined references, and basic consistency issues. \
            For each issue found, provide:\n\
            1. The line number or location\n\
            2. A description of the issue\n\
            3. The severity (Error, Warning, or Info)\n\
            4. A suggested fix\n\
            Format each issue as: \"Line [number]: [description] - [severity]\\nSuggestion: [fix]\"\n\n\
            After listing all issues, provide a final judgment: Is the specification syntax valid (true/false)?",
            spec.formal_spec.verification_language.to_string(),
            spec.formal_spec.spec_code
        );

        // Call the LLM API
        let response = self.call_llm_api(&prompt).await.map_err(AxiomError::from)?;

        // Parse the response to determine validity and extract issues
        let is_valid =
            response.to_lowercase().contains("the specification syntax is valid: true") ||
            response.to_lowercase().contains("is the specification syntax valid? true");

        // Extract issues
        let mut issues = Vec::new();
        let mut current_issue = String::new();
        let mut current_line = None;
        let mut current_severity = IssueSeverity::Info;
        let mut current_suggestion = None;

        for line in response.lines() {
            if line.starts_with("Line ") || line.starts_with("Location ") {
                // Save the previous issue if it exists
                if !current_issue.is_empty() {
                    issues.push(ValidationIssue {
                        severity: current_severity.clone(),
                        message: current_issue.clone(),
                        related_property: None,
                        line_number: current_line,
                        suggested_fix: current_suggestion.clone(),
                    });
                }

                // Parse the new issue
                current_issue = line.to_string();

                // Extract line number
                if let Some(start) = line.find("Line ") {
                    if let Some(end) = line[start + 5..].find(':') {
                        if
                            let Ok(line_num) = line[start + 5..start + 5 + end]
                                .trim()
                                .parse::<usize>()
                        {
                            current_line = Some(line_num);
                        }
                    }
                }

                // Extract severity
                if line.contains("Error") {
                    current_severity = IssueSeverity::Error;
                } else if line.contains("Warning") {
                    current_severity = IssueSeverity::Warning;
                } else {
                    current_severity = IssueSeverity::Info;
                }

                current_suggestion = None;
            } else if line.starts_with("Suggestion:") {
                current_suggestion = Some(
                    line.trim_start_matches("Suggestion:").trim().to_string()
                );
            }
        }

        // Add the last issue if it exists
        if !current_issue.is_empty() {
            issues.push(ValidationIssue {
                severity: current_severity,
                message: current_issue,
                related_property: None,
                line_number: current_line,
                suggested_fix: current_suggestion,
            });
        }

        Ok(ValidationReport {
            is_valid,
            issues,
            tool_validated: false,
            tool_output: None,
        })
    }

    async fn validate_type_checking(&self, spec: &Specification) -> AxiomResult<ValidationReport> {
        // Prepare the prompt for type checking validation
        let prompt = format!(
            "You are a formal verification expert with deep knowledge of {} type systems. \
            Perform type checking on this specification:\n\n\
            ```\n{}\n```\n\n\
            Check for type errors, type inconsistencies, and type-related issues. \
            For each issue found, provide:\n\
            1. The line number or location\n\
            2. A description of the type error\n\
            3. The severity (Error, Warning, or Info)\n\
            4. A suggested fix\n\
            Format each issue as: \"Line [number]: [description] - [severity]\\nSuggestion: [fix]\"\n\n\
            After listing all issues, provide a final judgment: Does the specification pass type checking (true/false)?",
            spec.formal_spec.verification_language.to_string(),
            spec.formal_spec.spec_code
        );

        // Call the LLM API
        let response = self.call_llm_api(&prompt).await.map_err(AxiomError::from)?;

        // Parse the response to determine validity and extract issues
        let is_valid =
            response.to_lowercase().contains("the specification passes type checking: true") ||
            response.to_lowercase().contains("does the specification pass type checking? true");

        // Extract issues (similar to validate_syntax)
        let mut issues = Vec::new();
        let mut current_issue = String::new();
        let mut current_line = None;
        let mut current_severity = IssueSeverity::Info;
        let mut current_suggestion = None;

        for line in response.lines() {
            if line.starts_with("Line ") || line.starts_with("Location ") {
                // Save the previous issue if it exists
                if !current_issue.is_empty() {
                    issues.push(ValidationIssue {
                        severity: current_severity.clone(),
                        message: current_issue.clone(),
                        related_property: None,
                        line_number: current_line,
                        suggested_fix: current_suggestion.clone(),
                    });
                }

                // Parse the new issue
                current_issue = line.to_string();

                // Extract line number
                if let Some(start) = line.find("Line ") {
                    if let Some(end) = line[start + 5..].find(':') {
                        if
                            let Ok(line_num) = line[start + 5..start + 5 + end]
                                .trim()
                                .parse::<usize>()
                        {
                            current_line = Some(line_num);
                        }
                    }
                }

                // Extract severity
                if line.contains("Error") {
                    current_severity = IssueSeverity::Error;
                } else if line.contains("Warning") {
                    current_severity = IssueSeverity::Warning;
                } else {
                    current_severity = IssueSeverity::Info;
                }

                current_suggestion = None;
            } else if line.starts_with("Suggestion:") {
                current_suggestion = Some(
                    line.trim_start_matches("Suggestion:").trim().to_string()
                );
            }
        }

        // Add the last issue if it exists
        if !current_issue.is_empty() {
            issues.push(ValidationIssue {
                severity: current_severity,
                message: current_issue,
                related_property: None,
                line_number: current_line,
                suggested_fix: current_suggestion,
            });
        }

        Ok(ValidationReport {
            is_valid,
            issues,
            tool_validated: false,
            tool_output: None,
        })
    }

    async fn validate_formal_verification(
        &self,
        spec: &Specification
    ) -> AxiomResult<ValidationReport> {
        // Prepare the prompt for formal verification validation
        let prompt = format!(
            "You are a formal verification expert with deep knowledge of {}. \
            Validate whether this specification can be formally verified:\n\n\
            ```\n{}\n```\n\n\
            Check for issues that would prevent successful verification, such as:\n\
            1. Incompleteness in definitions\n\
            2. Unprovable assertions or theorems\n\
            3. Missing lemmas or auxiliary functions\n\
            4. Inconsistent axioms\n\
            For each issue found, provide:\n\
            1. The line number or location\n\
            2. A description of the verification issue\n\
            3. The severity (Error, Warning, or Info)\n\
            4. A suggested fix\n\
            Format each issue as: \"Line [number]: [description] - [severity]\\nSuggestion: [fix]\"\n\n\
            After listing all issues, provide a final judgment: Can the specification be formally verified as written (true/false)?",
            spec.formal_spec.verification_language.to_string(),
            spec.formal_spec.spec_code
        );

        // Call the LLM API
        let response = self.call_llm_api(&prompt).await.map_err(AxiomError::from)?;

        // Parse the response to determine validity and extract issues
        let is_valid =
            response.to_lowercase().contains("the specification can be formally verified: true") ||
            response
                .to_lowercase()
                .contains("can the specification be formally verified as written? true");

        // Extract issues (similar to validate_syntax)
        let mut issues = Vec::new();
        let mut current_issue = String::new();
        let mut current_line = None;
        let mut current_severity = IssueSeverity::Info;
        let mut current_suggestion = None;

        for line in response.lines() {
            if line.starts_with("Line ") || line.starts_with("Location ") {
                // Save the previous issue if it exists
                if !current_issue.is_empty() {
                    issues.push(ValidationIssue {
                        severity: current_severity.clone(),
                        message: current_issue.clone(),
                        related_property: None,
                        line_number: current_line,
                        suggested_fix: current_suggestion.clone(),
                    });
                }

                // Parse the new issue
                current_issue = line.to_string();

                // Extract line number
                if let Some(start) = line.find("Line ") {
                    if let Some(end) = line[start + 5..].find(':') {
                        if
                            let Ok(line_num) = line[start + 5..start + 5 + end]
                                .trim()
                                .parse::<usize>()
                        {
                            current_line = Some(line_num);
                        }
                    }
                }

                // Extract severity
                if line.contains("Error") {
                    current_severity = IssueSeverity::Error;
                } else if line.contains("Warning") {
                    current_severity = IssueSeverity::Warning;
                } else {
                    current_severity = IssueSeverity::Info;
                }

                current_suggestion = None;
            } else if line.starts_with("Suggestion:") {
                current_suggestion = Some(
                    line.trim_start_matches("Suggestion:").trim().to_string()
                );
            }
        }

        // Add the last issue if it exists
        if !current_issue.is_empty() {
            issues.push(ValidationIssue {
                severity: current_severity,
                message: current_issue,
                related_property: None,
                line_number: current_line,
                suggested_fix: current_suggestion,
            });
        }

        Ok(ValidationReport {
            is_valid,
            issues,
            tool_validated: false,
            tool_output: Some(response),
        })
    }
}
// Implement to_string for Domain, VerificationLanguage, etc.
impl std::fmt::Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Domain::Cryptography => write!(f, "Cryptography"),
            Domain::DistributedSystems => write!(f, "Distributed Systems"),
            Domain::WebSecurity => write!(f, "Web Security"),
            Domain::MachineLearning => write!(f, "Machine Learning"),
            Domain::SystemsSoftware => write!(f, "Systems Software"),
            Domain::Blockchain => write!(f, "Blockchain"),
            Domain::SafetyControl => write!(f, "Safety Control"),
            Domain::HighAssuranceSoftware => write!(f, "High Assurance Software"),
            Domain::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl std::fmt::Display for VerificationLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerificationLanguage::FStarLang => write!(f, "F*"),
            VerificationLanguage::DafnyLang => write!(f, "Dafny"),
            VerificationLanguage::CoqLang => write!(f, "Coq"),
            VerificationLanguage::IsabelleLang => write!(f, "Isabelle"),
            VerificationLanguage::LeanLang => write!(f, "Lean"),
            VerificationLanguage::TLAPlus => write!(f, "TLA+"),
            VerificationLanguage::Why3Lang => write!(f, "Why3"),
            VerificationLanguage::Z3SMT => write!(f, "Z3 SMT"),
            VerificationLanguage::ACSL => write!(f, "ACSL"),
            VerificationLanguage::JML => write!(f, "JML"),
            VerificationLanguage::Liquid => write!(f, "Liquid Haskell"),
            VerificationLanguage::RustMIRAI => write!(f, "MIRAI"),
            VerificationLanguage::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl std::fmt::Display for crate::models::common::VerificationSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::models::common::VerificationSystem::FStar => write!(f, "F*"),
            crate::models::common::VerificationSystem::Dafny => write!(f, "Dafny"),
            crate::models::common::VerificationSystem::Coq => write!(f, "Coq"),
            crate::models::common::VerificationSystem::Isabelle => write!(f, "Isabelle"),
            crate::models::common::VerificationSystem::Lean => write!(f, "Lean"),
            crate::models::common::VerificationSystem::TLA => write!(f, "TLA+"),
            crate::models::common::VerificationSystem::Why3 => write!(f, "Why3"),
            crate::models::common::VerificationSystem::Z3 => write!(f, "Z3"),
            crate::models::common::VerificationSystem::Custom(name) => write!(f, "{}", name),
        }
    }
}
