# config-loadr

A type-safe configuration loading library that enforces better documentation on what each configuration option does.

## Why use this library?

Since the dawn of time, devops engineers have conducted archeological expeditions through sparsely documented codebases just to figure out how to configure the application. This library intends to enforce better documentation on what the different configuration options do in an ergonomic and type-safe way.

### Beautiful error messages

```
Configuration failed with 2 error(s):
  - ERROR_TEST_STRING: Is missing from environment and is required
        Description: Test value
        Example: ERROR_TEST_STRING=test

  - TEST_WRONG_TYPE: Invalid value 'not_an_int'
        Description: Test value
        Example: TEST_WRONG_TYPE=42
```

## Quick Start

```rust
use config_loadr::define_config;
use std::net::IpAddr;

define_config! {
    pub struct Config {
        // Required field - must be present in environment
        #[field(env = "SECRET", doc = "Secret key for authentication", example = "my-secret-key".to_string(), required)]
        pub secret: String,

        // Field with default - uses default if env var is missing
        #[field(env = "PORT", doc = "Port number for the HTTP server", default = 8080u16)]
        pub port: u16,

        // Optional field - returns None if missing, Error if invalid
        #[field(env = "API_KEY", doc = "Optional third-party API key", example = "key123".into(), optional)]
        pub api_key: Option<String>,

        // Can take any type that implements FromStr
        #[field(env = "LISTEN_ADDRESS", doc = "For example, an ip address", default = "0.0.0.0".parse::<IpAddr>().unwrap())]
        pub listen_address: IpAddr,
    }
}

fn main() {
    let config = Config::load();

    assert_eq!(config.port, 8080);           // ✓ Uses default value
    assert_eq!(config.api_key, None);        // ✓ Returns None when unset
    assert_eq!(config.secret, "my-secret");  // ✓ Loaded from environment
}
```

## Field Attributes

The `#[field(...)]` attribute supports three modes:

| Attribute | Type | Behavior | Example Required |
|-----------|------|----------|------------------|
| `required` | `T` | Must be set in environment, errors if missing | Yes |
| `default = <expr>` | `T` | Uses default if missing, errors if invalid | No |
| `optional` | `Option<T>` | Returns `None` if missing, errors if invalid | Optional |

All fields also support:
- `env = "VAR_NAME"` - Environment variable name (required)
- `doc = "description"` - Field description (required unless `#[allow(missing_docs)]` on struct)
- `example = value` - Example value for documentation

## Loading Methods

### Using `load()` - Panic on Error

```rust
use config_loadr::define_config;

define_config! {
    pub struct Config {
        #[field(env = "PORT", doc = "Server port", example = 8080u16, required)]
        pub port: u16,
    }
}

fn main() {
    // Panics with detailed error message if config is invalid
    // For example, if PORT=not-a-number or missing, it will return a ConfigError
    let config = Config::load(); // Panics on error
    assert_eq!(config.port, 8080);
}
```

### Using `new()` - Handle Errors Yourself

```rust
use config_loadr::define_config;

define_config! {
    pub struct Config {
        #[field(env = "PORT", doc = "Server port", default = 8080u16)]
        pub port: u16,

        #[field(env = "SECRET", doc = "Secret key", example = "secret".to_string(), required)]
        pub secret: String,
    }
}

fn main() {
    match Config::new() {
        Ok(config) => {
            assert_eq!(config.port, 8080);
            assert!(!config.secret.is_empty());
        },
        Err(errors) => {
            // Returns Vec<ConfigError> containing all errors
            assert!(errors.len() > 0);
            for error in errors {
                eprintln!("Config error: {}", error);
            }
        }
    }
}
```

## Optional Fields Behavior

Optional fields distinguish between **missing** (returns `None`) and **invalid** (returns `Error`):

```rust
use config_loadr::define_config;

define_config! {
    pub struct Config {
        #[field(env = "PORT", doc = "Optional port number", example = 8080, optional)]
        pub port: Option<u16>,
    }
}

// Case 1: Environment variable not set
std::env::remove_var("PORT");
let config = Config::new().unwrap();
assert_eq!(config.port, None);  // ✓ Returns None

// Case 2: Valid value in environment
std::env::set_var("PORT", "3000");
let config = Config::new().unwrap();
assert_eq!(config.port, Some(3000));  // ✓ Parses successfully

// Case 3: Invalid value in environment
std::env::set_var("PORT", "not-a-number");
let result = Config::new();
assert!(result.is_err());  // ✓ Returns error for invalid value
```

This behavior catches configuration mistakes while allowing truly optional values.

## Error Handling

The library collects **all** configuration errors before failing, not just the first one:

```rust
use config_loadr::define_config;

define_config! {
    pub struct Config {
        #[field(env = "DATABASE_URL", doc = "PostgreSQL connection string", example = "postgresql://localhost/db".into(), required)]
        pub database_url: String,

        #[field(env = "REDIS_URL", doc = "Redis connection string", example = "redis://localhost".into(), required)]
        pub redis_url: String,

        #[field(env = "PORT", doc = "Server port", default = 8080u16)]
        pub port: u16,
    }
}

// If DATABASE_URL and REDIS_URL are both missing:
let result = Config::new();
assert!(result.is_err());

if let Err(errors) = result {
    assert_eq!(errors.len(), 2);  // ✓ Both errors collected
    // Errors are formatted with colors and examples:
    // DATABASE_URL: Is missing from environment and is required
    //     Description: PostgreSQL connection string
    //     Example: DATABASE_URL=postgresql://localhost/db
    // REDIS_URL: Is missing from environment and is required
    //     Description: Redis connection string
    //     Example: REDIS_URL=redis://localhost
}
```

## Metadata Access

Access configuration metadata programmatically:

```rust
use config_loadr::define_config;

define_config! {
    pub struct Config {
        #[field(env = "PORT", doc = "Server port number", default = 8080u16)]
        pub port: u16,

        #[field(env = "DEBUG", doc = "Enable debug mode", default = false)]
        pub debug: bool,
    }
}

fn main() {
    let metadata = Config::metadata();

    // Access field metadata
    assert_eq!(metadata.port.key, "PORT");
    assert_eq!(metadata.port.description, "Server port number");
    assert_eq!(metadata.port.default, 8080);
    assert!(!metadata.port.required);

    assert_eq!(metadata.debug.key, "DEBUG");
    assert_eq!(metadata.debug.description, "Enable debug mode");
    assert_eq!(metadata.debug.default, false);
}
```

## Documentation Generation

Generate markdown documentation for your configuration:

```rust
use config_loadr::define_config;

define_config! {
    pub struct Config {
        #[field(env = "PORT", doc = "Server port", default = 8080u16)]
        pub port: u16,

        #[field(env = "SECRET", doc = "Secret key", example = "secret".to_string(), required)]
        pub secret: String,
    }
}

fn main() {
    let builder = Config::builder_for_docs();
    builder.write_docs("CONFIG.md").unwrap();

    // Generates a markdown table:
    // ## Environment Variables Summary
    //
    // | Variable | Required | Description | Default/Example |
    // |----------|----------|-------------|-----------------|
    // | PORT     | No       | Server port | 8080            |
    // | SECRET   | Yes      | Secret key  | secret          |
}
```

## Additional Features

### Environment Enum

The library provides a built-in `Environment` enum for common use cases:

```rust
use config_loadr::{define_config, Environment};

define_config! {
    pub struct Config {
        #[field(env = "ENVIRONMENT", doc = "Application environment (dev/prod)", default = Environment::Dev)]
        pub env: Environment,
    }
}

fn main() {
    let config = Config::load();

    if config.env.is_prod() {
        // Production-specific logic
        assert!(matches!(config.env, Environment::Prod));
    } else {
        // Development logic
        assert!(matches!(config.env, Environment::Dev));
    }
}
```

## Disclaimer

This library has been developed with the help of LLMs and is not intended for production use before v1.0.0.
