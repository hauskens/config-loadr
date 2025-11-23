use crate::error::ConfigError;
use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Environment {
    Prod,
    Dev,
}

impl FromStr for Environment {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "prod" | "production" => Ok(Self::Prod),
            "dev" | "development" => Ok(Self::Dev),
            _ => Err(ConfigError::InvalidEnvironment {
                key: "ENVIRONMENT".to_string(),
                value: s.to_string(),
                description: "Expected 'dev' or 'prod'".to_string(),
                example: Some("prod".to_string()),
            }),
        }
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Prod => write!(f, "prod"),
            Self::Dev => write!(f, "dev"),
        }
    }
}

impl Environment {
    pub fn is_prod(&self) -> bool {
        matches!(self, Self::Prod)
    }

    pub fn is_dev(&self) -> bool {
        matches!(self, Self::Dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_prod() {
        let env: Environment = "prod".parse().unwrap();
        assert_eq!(env, Environment::Prod);
    }

    #[test]
    fn test_parse_production() {
        let env: Environment = "production".parse().unwrap();
        assert_eq!(env, Environment::Prod);
    }

    #[test]
    fn test_parse_dev() {
        let env: Environment = "dev".parse().unwrap();
        assert_eq!(env, Environment::Dev);
    }

    #[test]
    fn test_parse_development() {
        let env: Environment = "development".parse().unwrap();
        assert_eq!(env, Environment::Dev);
    }

    #[test]
    fn test_parse_invalid() {
        let result: Result<Environment, ConfigError> = "staging".parse();
        assert!(result.is_err());

        if let Err(ConfigError::InvalidEnvironment {
            key,
            value,
            description,
            example,
        }) = result
        {
            assert_eq!(key, "ENVIRONMENT");
            assert_eq!(value, "staging");
            assert!(description.contains("Expected 'dev' or 'prod'"));
            assert_eq!(example, Some("prod".to_string()));
        } else {
            panic!("Expected InvalidEnvironment error");
        }
    }

    #[test]
    fn test_display_prod() {
        let env = Environment::Prod;
        assert_eq!(env.to_string(), "prod");
    }

    #[test]
    fn test_display_dev() {
        let env = Environment::Dev;
        assert_eq!(env.to_string(), "dev");
    }

    #[test]
    fn test_is_prod() {
        assert!(Environment::Prod.is_prod());
        assert!(!Environment::Dev.is_prod());
    }

    #[test]
    fn test_is_dev() {
        assert!(Environment::Dev.is_dev());
        assert!(!Environment::Prod.is_dev());
    }
}
