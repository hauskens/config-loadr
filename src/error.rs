use colored::Colorize;
use std::fmt;

/// Errors that can occur during configuration loading
#[derive(Debug, Clone)]
pub enum ConfigError {
    /// A required environment variable is missing
    MissingEnvVar {
        key: String,
        description: String,
        example: Option<String>,
    },
    /// An environment variable has an invalid value
    InvalidEnvironment {
        key: String,
        value: String,
        description: String,
        example: Option<String>,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingEnvVar {
                key,
                description,
                example,
            } => {
                writeln!(
                    f,
                    "{}: Is missing from environment and is required",
                    key.magenta().bold()
                )?;
                writeln!(f, "\tDescription: {}", description)?;
                if let Some(ex) = example {
                    writeln!(f, "\tExample: {}={}", key.magenta().bold(), ex.cyan())?;
                }
                Ok(())
            }
            ConfigError::InvalidEnvironment {
                key,
                value,
                description,
                example,
            } => {
                writeln!(
                    f,
                    "{}: Invalid value {}",
                    key.magenta().bold(),
                    format!("'{}'", value).red(),
                )?;
                writeln!(f, "\tDescription: {}", description)?;
                if let Some(ex) = example {
                    writeln!(f, "\tExample: {}={}", key.magenta().bold(), ex.cyan())?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_env_var_with_example() {
        colored::control::set_override(false);

        let error = ConfigError::MissingEnvVar {
            key: "DATABASE_URL".to_string(),
            description: "PostgreSQL connection string".to_string(),
            example: Some("postgresql://user:pass@localhost/db".to_string()),
        };

        let output = error.to_string();
        assert!(output.contains("DATABASE_URL:"));
        assert!(output.contains("Description: PostgreSQL connection string"));
        assert!(output.contains("Example: DATABASE_URL=postgresql://user:pass@localhost/db"));
    }

    #[test]
    fn test_missing_env_var_without_example() {
        colored::control::set_override(false);

        let error = ConfigError::MissingEnvVar {
            key: "SECRET_KEY".to_string(),
            description: "Secret encryption key".to_string(),
            example: None,
        };

        let output = error.to_string();
        assert!(output.contains("SECRET_KEY:"));
        assert!(output.contains("Description: Secret encryption key"));
        assert!(!output.contains("Example:"));
    }

    #[test]
    fn test_invalid_environment() {
        colored::control::set_override(false);

        let error = ConfigError::InvalidEnvironment {
            key: "PORT".to_string(),
            value: "not-a-number".to_string(),
            description: "Must be a valid port number".to_string(),
            example: Some("8080".to_string()),
        };

        let output = error.to_string();
        assert!(output.contains("PORT"));
        assert!(output.contains("Invalid value 'not-a-number'"));
        assert!(output.contains("Must be a valid port number"));
        assert!(output.contains("Example: PORT=8080"));
    }

    #[test]
    fn test_clone() {
        let error1 = ConfigError::MissingEnvVar {
            key: "TEST".to_string(),
            description: "Test var".to_string(),
            example: Some("example".to_string()),
        };

        let error2 = error1.clone();

        assert_eq!(error1.to_string(), error2.to_string());
    }

    #[test]
    fn test_debug_format() {
        let error = ConfigError::InvalidEnvironment {
            key: "ENV".to_string(),
            value: "test".to_string(),
            description: "Invalid environment".to_string(),
            example: None,
        };

        let debug_output = format!("{:?}", error);
        assert!(debug_output.contains("InvalidEnvironment"));
        assert!(debug_output.contains("ENV"));
    }

    #[test]
    fn test_invalid_environment_without_example() {
        colored::control::set_override(false);

        let error = ConfigError::InvalidEnvironment {
            key: "SECRET".to_string(),
            value: "bad-value".to_string(),
            description: "Must be valid format".to_string(),
            example: None,
        };

        let output = error.to_string();
        assert!(output.contains("SECRET"));
        assert!(output.contains("Invalid value 'bad-value'"));
        assert!(output.contains("Must be valid format"));
        assert!(!output.contains("Example:"));
    }
}
