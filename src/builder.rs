use crate::error::ConfigError;
use colored::Colorize;
use std::{env, fs, path::Path, str::FromStr};

/// Metadata about a configuration field for documentation generation
#[derive(Debug, Clone)]
pub struct FieldMetadata {
    /// Environment variable key
    pub key: String,
    /// Human-readable description
    pub description: String,
    /// Default or example value as a string
    pub default_str: String,
    /// Whether this field is required
    pub required: bool,
}

/// Parses an environment variable into a specific type
pub fn env_parse<'a, T: FromStr>(
    key: &str,
    description: &str,
    example: impl Into<Option<&'a str>>,
) -> Result<T, ConfigError> {
    let example = example.into();
    match env::var(key) {
        Ok(s) => match s.parse() {
            Ok(parsed) => Ok(parsed),
            Err(_) => Err(ConfigError::InvalidEnvironment {
                key: key.to_string(),
                value: s,
                description: description.to_string(),
                example: example.map(|s| s.to_string()),
            }),
        },
        Err(_) => Err(ConfigError::MissingEnvVar {
            key: key.to_string(),
            description: description.to_string(),
            example: example.map(|s| s.to_string()),
        }),
    }
}

/// Loads a required environment variable, returning an error if missing or invalid
pub fn env_required<T: FromStr + std::fmt::Display + Clone>(
    key: &'static str,
    description: &'static str,
    example: T,
) -> Result<T, ConfigError> {
    let example_str = example.to_string();
    env_parse(key, description, example_str.as_ref())
}

/// Loads an optional environment variable with a default value
///
/// Returns the default value only if the environment variable is missing.
/// If the environment variable exists but cannot be parsed, returns an error.
pub fn env_or_default<T: FromStr + std::fmt::Display + Clone>(
    key: &'static str,
    description: &'static str,
    default: T,
) -> Result<T, ConfigError> {
    let default_str = default.to_string();
    match env_parse(key, description, Some(default_str.as_str())) {
        Ok(value) => Ok(value),
        Err(ConfigError::MissingEnvVar { .. }) => Ok(default),
        Err(e) => Err(e), // Propagate InvalidEnvironment and other errors
    }
}

/// Load an optional environment variable, returning None if missing
///
/// Returns Ok(None) if the environment variable is not set.
/// Returns Ok(Some(value)) if the environment variable is set and parses successfully.
/// Returns Err if the environment variable is set but cannot be parsed.
pub fn env_or_option<T: FromStr>(
    key: &'static str,
    description: &'static str,
    example: impl Into<Option<&'static str>>,
) -> Result<Option<T>, ConfigError> {
    let example_str = example.into();
    match env_parse(key, description, example_str) {
        Ok(value) => Ok(Some(value)),
        Err(ConfigError::MissingEnvVar { .. }) => Ok(None),
        Err(e) => Err(e), // Propagate parse errors
    }
}

/// Helper to format multiple configuration errors into a panic message
pub fn format_config_errors(errors: &[ConfigError]) -> String {
    let error_summary = errors
        .iter()
        .map(|e| format!("  - {}", e))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "Configuration failed with {} error(s):\n{}",
        errors.len().to_string().yellow().bold(),
        error_summary
    )
}

/// A builder pattern for loading configuration with error collection
///
/// # Example
/// ```rust
/// use config_loadr::ConfigBuilder;
///
/// let mut builder = ConfigBuilder::new();
/// let port = builder.required::<u16>("PORT", "Server port", 8080);
///
/// builder.validate();
/// ```
pub struct ConfigBuilder {
    errors: Vec<ConfigError>,
    fields: Vec<FieldMetadata>,
}

impl ConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            fields: Vec::new(),
        }
    }

    /// Load a required field, collecting errors if it fails
    pub fn required<T: FromStr + std::fmt::Display + Clone>(
        &mut self,
        key: &'static str,
        description: &'static str,
        example: T,
    ) -> Option<T> {
        // Capture metadata
        self.fields.push(FieldMetadata {
            key: key.to_string(),
            description: description.to_string(),
            default_str: example.to_string(),
            required: true,
        });

        match env_required(key, description, example) {
            Ok(value) => Some(value),
            Err(e) => {
                self.errors.push(e);
                None
            }
        }
    }

    /// Load a field, fallback to default value if missing
    ///
    /// Returns None and collects the error if the environment variable exists but is invalid.
    pub fn or_default<T: FromStr + std::fmt::Display + Clone>(
        &mut self,
        key: &'static str,
        description: &'static str,
        default: T,
    ) -> Option<T> {
        // Capture metadata
        self.fields.push(FieldMetadata {
            key: key.to_string(),
            description: description.to_string(),
            default_str: default.to_string(),
            required: false,
        });

        match env_or_default(key, description, default) {
            Ok(value) => Some(value),
            Err(e) => {
                self.errors.push(e);
                None
            }
        }
    }

    /// Load an optional field that may be None
    ///
    /// Returns None if the environment variable is not set, or Some(value) if it is.
    /// Collects an error if the environment variable is set but cannot be parsed.
    pub fn optional<T: FromStr>(
        &mut self,
        key: &'static str,
        description: &'static str,
        example: impl Into<Option<&'static str>>,
    ) -> Option<T> {
        let example_str = example.into();

        // Capture metadata
        self.fields.push(FieldMetadata {
            key: key.to_string(),
            description: description.to_string(),
            default_str: example_str.unwrap_or("").to_string(),
            required: false,
        });

        match env_or_option(key, description, example_str) {
            Ok(value) => value,
            Err(e) => {
                self.errors.push(e);
                None
            }
        }
    }

    /// Validate that all configuration fields loaded successfully
    ///
    /// Unlike `finish()`, this doesn't consume the builder, allowing you to call
    /// `write_docs()` afterward.
    pub fn validate(&self) -> Result<(), Vec<ConfigError>> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    /// Finish building and return any errors that were collected
    ///
    /// This method consumes the builder. Use `validate()` if you need to keep
    /// the builder for calling `write_docs()`.
    pub fn finish(self) -> Result<(), Vec<ConfigError>> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors)
        }
    }

    /// Finish building and panic if there were any errors
    pub fn finish_or_panic(self) {
        if !self.errors.is_empty() {
            panic!("{}", format_config_errors(&self.errors));
        }
    }

    /// Write configuration documentation to a markdown file
    ///
    /// Generates a markdown file documenting all configuration fields that were
    /// registered with this builder, including their descriptions, types, and
    /// default/example values.
    ///
    /// # Example
    /// ```no_run
    /// use config_loadr::ConfigBuilder;
    ///
    /// let mut builder = ConfigBuilder::new();
    /// let port = builder.or_default("PORT", "Server port", 8080);
    /// builder.validate().ok();
    /// builder.write_docs("CONFIG.md").unwrap();
    /// ```
    pub fn write_docs(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let mut md = String::new();

        // Summary table
        md.push_str("## Environment Variables Summary\n\n");
        md.push_str("| Variable | Required | Description | Default/Example |\n");
        md.push_str("|----------|----------|-------------|------------------|\n");
        for field in &self.fields {
            let required_str = if field.required { "Yes" } else { "No" };
            let default_display = if field.default_str.is_empty() {
                "-".to_string()
            } else {
                field.default_str.clone()
            };
            md.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                field.key, required_str, field.description, default_display
            ));
        }

        fs::write(path, md)
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::ConfigFieldMeta;

    #[test]
    fn test_builder_new() {
        let builder = ConfigBuilder::new();
        assert_eq!(builder.errors.len(), 0);
    }

    #[test]
    fn test_builder_default() {
        let builder = ConfigBuilder::default();
        assert_eq!(builder.errors.len(), 0);
    }

    #[test]
    fn test_builder_finish_with_no_errors() {
        let builder = ConfigBuilder::new();
        let result = builder.finish();
        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_collects_errors() {
        let mut builder = ConfigBuilder::new();

        // Manually add errors to simulate failed required fields
        builder.errors.push(ConfigError::MissingEnvVar {
            key: "MISSING_VAR".to_string(),
            description: "Test variable".to_string(),
            example: None,
        });

        let result = builder.finish();
        assert!(result.is_err());

        if let Err(errors) = result {
            assert_eq!(errors.len(), 1);
        }
    }

    #[test]
    fn test_builder_collects_multiple_errors() {
        let mut builder = ConfigBuilder::new();

        builder.errors.push(ConfigError::MissingEnvVar {
            key: "VAR1".to_string(),
            description: "First variable".to_string(),
            example: None,
        });

        builder.errors.push(ConfigError::MissingEnvVar {
            key: "VAR2".to_string(),
            description: "Second variable".to_string(),
            example: None,
        });

        let result = builder.finish();
        assert!(result.is_err());

        if let Err(errors) = result {
            assert_eq!(errors.len(), 2);
        }
    }

    #[test]
    fn test_format_config_errors_single() {
        colored::control::set_override(false);

        let errors = vec![ConfigError::MissingEnvVar {
            key: "TEST_VAR".to_string(),
            description: "Test variable".to_string(),
            example: Some("example".to_string()),
        }];

        let formatted = format_config_errors(&errors);
        assert!(formatted.contains("Configuration failed with 1 error(s)"));
        assert!(formatted.contains("TEST_VAR"));
    }

    #[test]
    fn test_format_config_errors_multiple() {
        colored::control::set_override(false);

        let errors = vec![
            ConfigError::MissingEnvVar {
                key: "VAR1".to_string(),
                description: "First".to_string(),
                example: None,
            },
            ConfigError::InvalidEnvironment {
                key: "VAR2".to_string(),
                value: "bad".to_string(),
                description: "Second".to_string(),
                example: None,
            },
        ];

        let formatted = format_config_errors(&errors);
        assert!(formatted.contains("Configuration failed with 2 error(s)"));
        assert!(formatted.contains("VAR1"));
        assert!(formatted.contains("VAR2"));
    }

    #[test]
    fn test_finish_with_errors() {
        let mut builder = ConfigBuilder::new();

        builder.errors.push(ConfigError::MissingEnvVar {
            key: "MISSING".to_string(),
            description: "Test".to_string(),
            example: None,
        });

        // Test that errors are present instead of testing panic
        assert_eq!(builder.errors.len(), 1);
        assert!(matches!(
            builder.errors[0],
            ConfigError::MissingEnvVar { .. }
        ));
    }

    #[test]
    fn test_finish_or_panic_succeeds() {
        let builder = ConfigBuilder::new();
        builder.finish_or_panic();
    }

    #[test]
    fn test_config_field_meta_preserved() {
        // Test that ConfigFieldMeta has correct metadata structure
        let field = ConfigFieldMeta::required("TEST_KEY", "Test description", 123);

        assert_eq!(field.key, "TEST_KEY");
        assert_eq!(field.description, "Test description");
        assert_eq!(field.default, 123);
        assert!(field.required);
    }

    #[test]
    fn test_optional_field_meta() {
        let field = ConfigFieldMeta::optional("OPT_KEY", "Optional key", "default");

        assert_eq!(field.key, "OPT_KEY");
        assert_eq!(field.description, "Optional key");
        assert_eq!(field.default, "default");
        assert!(!field.required);
    }

    #[test]
    fn test_builder_captures_required_field_metadata() {
        let mut builder = ConfigBuilder::new();
        let _ = builder.required("TEST_KEY", "Test description", 123);

        assert_eq!(builder.fields.len(), 1);
        assert_eq!(builder.fields[0].key, "TEST_KEY");
        assert_eq!(builder.fields[0].description, "Test description");
        assert_eq!(builder.fields[0].default_str, "123");
        assert!(builder.fields[0].required);
    }

    #[test]
    fn test_builder_captures_optional_field_metadata() {
        let mut builder = ConfigBuilder::new();
        let _ = builder.or_default("PORT", "Server port", 8080);

        assert_eq!(builder.fields.len(), 1);
        assert_eq!(builder.fields[0].key, "PORT");
        assert_eq!(builder.fields[0].description, "Server port");
        assert_eq!(builder.fields[0].default_str, "8080");
        assert!(!builder.fields[0].required);
    }

    #[test]
    fn test_builder_captures_multiple_fields() {
        let mut builder = ConfigBuilder::new();
        let _ = builder.required("KEY1", "First key", "value1".to_string());
        let _ = builder.or_default("KEY2", "Second key", 42);
        let _ = builder.optional::<String>("KEY3", "Third key", Some("example"));

        assert_eq!(builder.fields.len(), 3);
        assert_eq!(builder.fields[0].key, "KEY1");
        assert_eq!(builder.fields[1].key, "KEY2");
        assert_eq!(builder.fields[2].key, "KEY3");
    }

    #[test]
    fn test_validate_does_not_consume_builder() {
        let builder = ConfigBuilder::new();
        let result = builder.validate();
        assert!(result.is_ok());
        // Builder still accessible here
        assert_eq!(builder.fields.len(), 0);
    }
}
