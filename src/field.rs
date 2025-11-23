use std::ops::Deref;

/// A configuration field with metadata and value
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConfigField<T> {
    /// Environment variable key
    pub key: &'static str,
    /// Human-readable description of what this config does
    pub description: &'static str,
    /// Default value for optional fields, will be displayed as example if required (true)
    pub default: T,
    /// Whether this field is required (true) or optional with a default (false)
    pub required: bool,
    /// The actual configuration value
    pub value: T,
}

impl<T> ConfigField<T> {
    pub fn required(key: &'static str, description: &'static str, example: T, value: T) -> Self {
        Self {
            key,
            description,
            default: example,
            required: true,
            value,
        }
    }

    pub fn optional(key: &'static str, description: &'static str, default: T, value: T) -> Self {
        Self {
            key,
            description,
            default,
            required: false,
            value,
        }
    }
}

// Allow using ConfigField<T> as &T without writing .value
impl<T> Deref for ConfigField<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

// Allow using ConfigField<T> as &T via AsRef
impl<T> AsRef<T> for ConfigField<T> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_field_creation() {
        let field = ConfigField::required("PORT", "Server port", 8080, 8080);

        assert_eq!(field.key, "PORT");
        assert_eq!(field.description, "Server port");
        assert_eq!(field.default, 8080);
        assert!(field.required);
        assert_eq!(field.value, 8080);
        assert_eq!(*field, 8080);
    }

    #[test]
    fn test_optional_field_creation() {
        let field = ConfigField::optional("PORT", "Server port", 8080, 8080);

        assert_eq!(field.key, "PORT");
        assert_eq!(field.description, "Server port");
        assert_eq!(field.default, 8080);
        assert!(!field.required);
        assert_eq!(field.value, 8080);
        assert_eq!(*field, 8080);
    }

    #[test]
    fn test_optional_field_creation_type_preserved_int() {
        let field = ConfigField::optional("PORT", "Server port", 1234, 8080);

        assert_eq!(field.key, "PORT");
        assert_eq!(field.description, "Server port");
        assert_eq!(field.default, 1234);
        assert!(!field.required);
        assert_eq!(field.value, 8080);
        assert_eq!(*field, 8080);
    }

    #[test]
    fn test_optional_field_creation_type_preserved_str() {
        let field = ConfigField::optional("PORT", "Server port", "1234", "8080");

        assert_eq!(field.key, "PORT");
        assert_eq!(field.description, "Server port");
        assert_eq!(field.default, "1234");
        assert!(!field.required);
        assert_eq!(field.value, "8080");
        assert_eq!(*field, "8080");
    }

    #[test]
    fn test_optional_field_creation_type_preserved_bool() {
        let field = ConfigField::optional("PORT", "Server port", false, true);

        assert_eq!(field.key, "PORT");
        assert_eq!(field.description, "Server port");
        assert!(!field.default);
        assert!(!field.required);
        assert!(field.value);
        assert!(*field);
    }

    #[test]
    fn test_field_without_example() {
        let field = ConfigField::required("SECRET", "Secret key", "secret-value", "secret-value");

        assert_eq!(field.default, "secret-value");
        assert_eq!(field.value, "secret-value");
        assert_eq!(*field, "secret-value");
    }

    #[test]
    fn test_deref_implementation() {
        let field = ConfigField::required("PORT", "Server port", 8080, 8080);

        assert_eq!(*field, 8080);

        let doubled = *field * 2;
        assert_eq!(doubled, 16160);
    }

    #[test]
    fn test_as_ref_implementation() {
        let field = ConfigField::required("NAME", "Service name", "my-service", "test-service");

        let name_ref: &str = field.as_ref();
        assert_eq!(name_ref, "test-service");
    }
}
