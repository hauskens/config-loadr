/// Metadata for a configuration field (without the value)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConfigFieldMeta<T> {
    /// Environment variable key
    pub key: &'static str,
    /// Human-readable description of what this config does
    pub description: &'static str,
    /// Default value for optional fields, will be displayed as example if required (true)
    pub default: T,
    /// Whether this field is required (true) or optional with a default (false)
    pub required: bool,
}

impl<T> ConfigFieldMeta<T> {
    pub fn required(key: &'static str, description: &'static str, example: T) -> Self {
        Self {
            key,
            description,
            default: example,
            required: true,
        }
    }

    pub fn optional(key: &'static str, description: &'static str, default: T) -> Self {
        Self {
            key,
            description,
            default,
            required: false,
        }
    }
}

// Re-export as ConfigField for backwards compatibility with macro internals
// This will be used only for metadata, not values
pub type ConfigField<T> = ConfigFieldMeta<T>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_field_meta_creation() {
        let field = ConfigFieldMeta::required("PORT", "Server port", 8080);

        assert_eq!(field.key, "PORT");
        assert_eq!(field.description, "Server port");
        assert_eq!(field.default, 8080);
        assert!(field.required);
    }

    #[test]
    fn test_optional_field_meta_creation() {
        let field = ConfigFieldMeta::optional("PORT", "Server port", 8080);

        assert_eq!(field.key, "PORT");
        assert_eq!(field.description, "Server port");
        assert_eq!(field.default, 8080);
        assert!(!field.required);
    }

    #[test]
    fn test_optional_field_meta_type_preserved_int() {
        let field = ConfigFieldMeta::optional("PORT", "Server port", 1234);

        assert_eq!(field.key, "PORT");
        assert_eq!(field.description, "Server port");
        assert_eq!(field.default, 1234);
        assert!(!field.required);
    }

    #[test]
    fn test_optional_field_meta_type_preserved_str() {
        let field = ConfigFieldMeta::optional("HOST", "Server host", "localhost");

        assert_eq!(field.key, "HOST");
        assert_eq!(field.description, "Server host");
        assert_eq!(field.default, "localhost");
        assert!(!field.required);
    }

    #[test]
    fn test_optional_field_meta_type_preserved_bool() {
        let field = ConfigFieldMeta::optional("DEBUG", "Debug mode", false);

        assert_eq!(field.key, "DEBUG");
        assert_eq!(field.description, "Debug mode");
        assert!(!field.default);
        assert!(!field.required);
    }

    #[test]
    fn test_required_field_with_example() {
        let field = ConfigFieldMeta::required("SECRET", "Secret key", "example-secret");

        assert_eq!(field.key, "SECRET");
        assert_eq!(field.default, "example-secret");
        assert!(field.required);
    }
}
