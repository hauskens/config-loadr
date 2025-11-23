# config-loadr

A type-safe, configuration loading library intended provide better documentation on what the different configuration options do.

## Disclaimer

This library has been coded with the help from LLM, and is not intended to be used as a production library before v1.0.0

## Features

- **Type-safe**: Uses Rust's type system to ensure configuration values are valid
- **Helpful errors**: Clear error messages with descriptions and examples
- **Metadata tracking**: Each config field includes key, description, and example
- **Flexible loading**: Support for required, optional, and default values
- **Error collection**: Load all config at once and collect all errors
- **Dotenv support**: Automatically loads .env files

## Basic Usage

```rust
use config_loadr::{ConfigField, env_or_default, env_required};

// Load a required config value with an example
let database_url = env_required::<String>(
    "DATABASE_URL",
    "PostgreSQL connection string",
    Some("postgresql://user:password@localhost/db".to_string()),
)?;

// Load an optional value with a default (default must be the correct type)
// Returns an error if the env var exists but can't be parsed
let port = env_or_default(
    "PORT",
    "Server port",
    8080u16,  // Default value is typed - used as default AND shown in errors!
)?;

// Access the value directly (ConfigField implements Deref)
println!("Server running on port {}", *port);
```

## Using the ConfigBuilder

For complex configurations with many fields, use `ConfigBuilder` to collect all errors and track metadata:

```rust
use config_loadr::ConfigBuilder;

// Define your config struct
struct AppConfig {
    pub database_url: ConfigField<String>,
    pub port: ConfigField<u16>,
    pub api_key: ConfigField<Option<String>>,
}

impl AppConfig {
    fn load() -> Result<(Self, ConfigBuilder), Vec<ConfigError>> {
        let mut builder = ConfigBuilder::new();

        let database_url = builder.required(
            "DATABASE_URL",
            "PostgreSQL connection string",
            "postgresql://user:pass@localhost/db".to_string(),
        );
        let port = builder.or_default("PORT", "Server port", 8080);
        let api_key = builder.optional::<String>("API_KEY", "Optional API key", Some("sk-..."));

        builder.validate()?;

        Ok((Self {
            database_url: database_url.unwrap(),
            port: port.unwrap(),
            api_key,
        }, builder))
    }
}

// Load config and get builder reference
let (config, builder) = AppConfig::load()?;

// Access values using Deref
println!("Database URL: {}", *config.database_url);
println!("Port: {}", *config.port);

// Generate documentation
builder.write_docs("CONFIG.md")?;
```

### Automatic Documentation Generation

The `write_docs()` method automatically generates markdown documentation for all configuration fields:

```rust
let mut builder = ConfigBuilder::new();
let database_url = builder.required("DATABASE_URL", "PostgreSQL connection", "postgresql://...".to_string());
let port = builder.or_default("PORT", "Server port", 8080);

builder.write_docs("CONFIG.md")?;
```

This generates a markdown file with:
- Separate sections for required and optional fields
- Descriptions and examples/defaults for each field
- A summary table of all environment variables

**Example output:**

```markdown
# Configuration Reference

## Required Fields

### DATABASE_URL
- **Description:** PostgreSQL connection
- **Example:** `DATABASE_URL=postgresql://...`

## Optional Fields

### PORT
- **Description:** Server port
- **Default:** `8080`

## Environment Variables Summary

| Variable | Required | Default/Example |
|----------|----------|-----------------|
| DATABASE_URL | Yes | postgresql://... |
| PORT | No | 8080 |
```

## Environment Enum

The library provides a built-in `Environment` enum for common runtime environments:

```rust
use config_loadr::{Environment, env_required};

let env = env_required::<Environment>(
    "ENVIRONMENT",
    "Runtime environment",
    Some(Environment::Prod),
)?;

if env.value.is_prod() {
    // Production-specific logic
}
```

Supported values: `dev`, `development`, `staging`, `stage`, `prod`, `production`

## ConfigField Structure

Each configuration field contains:

```rust
pub struct ConfigField<T> {
    pub key: &'static str,        // Environment variable name
    pub description: &'static str, // Human-readable description
    pub default: Option<T>,        // Default/example value (typed!)
    pub required: bool,            // Whether this field is required
    pub value: T,                  // The actual value
}
```

The `default` field serves dual purposes:
- **Required fields**: Used as an example in error messages
- **Optional fields**: Used as the actual default value AND shown in parse errors

You can create fields manually:

```rust
use config_loadr::ConfigField;

// Required field with example
let field = ConfigField::required(
    "API_KEY",
    "Authentication key",
    Some("sk-example-key".to_string()),
    "actual-key-value".to_string(),
);

// Optional field with default
let optional_field = ConfigField::optional(
    "DEBUG",
    "Enable debug mode",
    Some(false),
    false,
);

// If you don't want to provide an example/default, use None
let field_no_example = ConfigField::required(
    "SECRET_KEY",
    "Secret key for encryption",
    None,
    "secret-value".to_string(),
);
```

## Error Handling

The library provides detailed error messages:

**Missing environment variable:**
```
DATABASE_URL:
    Description: PostgreSQL connection string
    Example: DATABASE_URL=postgresql://user:password@localhost/db
```

**Invalid value (with example):**
```
PORT: Invalid value 'abc'. Must be a valid port number. Example: PORT=8080
```

## Features

- `serde`: Enable serialization/deserialization support for ConfigField and Environment

```toml
[dependencies]
config-loadr = { version = "*", features = ["serde"] }
```

## License

This library is part of the YappR project.
