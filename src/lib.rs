pub mod builder;
pub mod environment;
pub mod error;
pub mod field;
pub mod macros;

// Re-export main types
pub use builder::{ConfigBuilder, env_or_default, env_or_option, env_parse, env_required};
pub use environment::Environment;
pub use error::ConfigError;
pub use field::{ConfigField, ConfigFieldMeta};

// Re-export macro
pub use config_loadr_macros::define_config;

/// Trait for loading configuration from environment variables
pub trait Load: Sized {
    /// Load configuration from environment, panicking on validation errors
    fn load() -> Self;

    /// Load configuration from environment, returning errors instead of panicking
    fn new() -> Result<Self, Vec<ConfigError>>;

    /// Create a builder for documentation generation (without loading values)
    fn builder_for_docs() -> ConfigBuilder;
}
