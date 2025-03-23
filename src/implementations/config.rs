use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    FileReadError(#[from] std::io::Error),
    
    #[error("Failed to parse config file: {0}")]
    ParseError(#[from] serde_yaml::Error),
    
    #[error("Missing required API key: {0}")]
    MissingApiKey(String),
    
    #[error("Environment variable not found: {0}")]
    EnvVarNotFound(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiConfig {
    /// API key for LLM service
    pub api_key: Option<String>,
    
    /// API endpoint for LLM service
    pub api_endpoint: Option<String>,
    
    /// API model to use
    pub model: Option<String>,
    
    /// API organization ID (if applicable)
    pub organization_id: Option<String>,
    
    /// Additional API parameters
    pub parameters: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeneratorConfig {
    /// Configuration for the LLM used for natural language to formal spec conversion
    pub llm_api: ApiConfig,
    
    /// Configuration for verification tooling APIs (if any)
    pub verification_apis: HashMap<String, ApiConfig>,
    
    /// Default templates directory
    pub templates_dir: Option<String>,
    
    /// Default prompt templates
    pub prompt_templates: HashMap<String, String>,
    
    /// Use chain-of-thought reasoning for improved accuracy
    pub use_chain_of_thought: Option<bool>,
    
    /// Maximum tokens for API calls
    pub max_tokens: Option<usize>,
    
    /// Temperature for generation (0.0-1.0)
    pub temperature: Option<f32>,
    
    /// Custom domains and their configurations
    pub domain_configs: Option<HashMap<String, DomainConfig>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DomainConfig {
    /// Domain-specific templates
    pub templates: Option<Vec<String>>,
    
    /// Domain-specific prompt additions
    pub prompt_additions: Option<String>,
    
    /// Recommended verification systems for this domain
    pub recommended_verification_systems: Option<Vec<String>>,
    
    /// Recommended verification languages for this domain
    pub recommended_verification_languages: Option<Vec<String>>,
}

impl GeneratorConfig {
    /// Load configuration from a YAML file
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path)?;
        let config: GeneratorConfig = serde_yaml::from_str(&contents)?;
        Ok(config)
    }
    
    /// Get the API key, checking environment variables if not in config
    /// If the specified provider key is not found, it will try other providers
    pub fn get_api_key(&self, preferred_provider: &str) -> Result<(String, String), ConfigError> {
        use log::{debug, info};
        
        // First check if we have the API key in the config
        if let Some(api_key) = &self.llm_api.api_key {
            debug!("Using API key from config");
            return Ok((preferred_provider.to_string(), api_key.clone()));
        }
        
        // List of providers to check
        let providers = vec![
            ("openai", "OPENAI_API_KEY"),
            ("anthropic", "ANTHROPIC_API_KEY"),
            ("azure", "AZURE_OPENAI_API_KEY"),
            ("mistral", "MISTRAL_API_KEY"),
            ("together", "TOGETHER_API_KEY"),
        ];
        
        // Check for preferred provider first
        let env_var_name = match preferred_provider.to_lowercase().as_str() {
            "openai" => "OPENAI_API_KEY",
            "anthropic" => "ANTHROPIC_API_KEY",
            "azure" => "AZURE_OPENAI_API_KEY",
            "mistral" => "MISTRAL_API_KEY",
            "together" => "TOGETHER_API_KEY",
            _ => {
                debug!("Unknown provider: {}, will try known providers", preferred_provider);
                "UNKNOWN"
            },
        };
        
        if env_var_name != "UNKNOWN" {
            match std::env::var(env_var_name) {
                Ok(key) => {
                    info!("Using preferred provider: {}", preferred_provider);
                    return Ok((preferred_provider.to_string(), key));
                },
                Err(_) => {
                    debug!("Preferred provider {} not available, trying others", preferred_provider);
                }
            }
        }
        
        // Try all other providers
        for (provider, env_var) in providers {
            if provider != preferred_provider.to_lowercase() {
                match std::env::var(env_var) {
                    Ok(key) => {
                        info!("Using alternative provider: {} (preferred was {})", provider, preferred_provider);
                        return Ok((provider.to_string(), key));
                    },
                    Err(_) => {
                        debug!("Provider {} not available", provider);
                    }
                }
            }
        }
        
        // If we get here, no API keys were found
        Err(ConfigError::MissingApiKey("No API keys found for any provider".to_string()))
    }
    
    /// Get a domain-specific configuration, or return a default
    pub fn get_domain_config(&self, domain: &str) -> DomainConfig {
        if let Some(domain_configs) = &self.domain_configs {
            if let Some(config) = domain_configs.get(domain) {
                return config.clone();
            }
        }
        
        // Return default config if not found
        DomainConfig {
            templates: None,
            prompt_additions: None,
            recommended_verification_systems: None,
            recommended_verification_languages: None,
        }
    }
    
    /// Get the template for a specific task
    pub fn get_template(&self, template_name: &str) -> Option<String> {
        self.prompt_templates.get(template_name).cloned()
    }
}

/// Default configuration
impl Default for GeneratorConfig {
    fn default() -> Self {
        let mut prompt_templates = HashMap::new();
        prompt_templates.insert(
            "specification".to_string(),
            r#"
You are a formal verification expert. Your task is to translate natural language requirements into
formal specifications in the {{verification_language}} verification language.

Given the following requirements for a {{domain}} system:

{{requirements}}

Generate a complete, formal specification in {{verification_language}} that captures all the 
requirements and ensures correctness, safety, and security properties. Be thorough and precise.

The specification should include:
1. All necessary types and functions
2. Formal properties that must be satisfied
3. Preconditions and postconditions
4. Invariants that must be maintained
5. Security properties (if applicable)
6. Resource usage constraints (if applicable)

Additional context for this domain:
{{domain_context}}
"#.to_string(),
        );
        
        GeneratorConfig {
            llm_api: ApiConfig {
                api_key: None,
                api_endpoint: Some("https://api.openai.com/v1/chat/completions".to_string()),
                model: Some("gpt-4o".to_string()),
                organization_id: None,
                parameters: None,
            },
            verification_apis: HashMap::new(),
            templates_dir: None,
            prompt_templates,
            use_chain_of_thought: Some(true),
            max_tokens: Some(4096),
            temperature: Some(0.2),
            domain_configs: None,
        }
    }
}