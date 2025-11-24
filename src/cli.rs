use config_loadr::define_config;

define_config!(
    #[derive(Debug)]
    pub struct WorkingConfig {
        #[field(env = "TEST_STRING", doc = "Test value", example = "test".to_string(), required)]
        pub test_string: String,

        #[field(env = "TEST_INT", doc = "Test value", default = 123i32)]
        pub test_int: i32,

        #[field(env = "TEST_BOOL", doc = "Test value", default = true)]
        pub test_bool: bool,
        #[field(env = "TEST_OPTIONAL", doc = "Test value", example = 123, optional)]
        pub test_optional: Option<i32>,
    }
);

define_config!(
    #[derive(Debug)]
    pub struct ErrorConfig {
        #[field(env = "ERROR_TEST_STRING", doc = "Test value", example = "test".to_string(), required)]
        pub test_string: String,

        #[field(env = "ERROR_TEST_INT", doc = "Test value", default = 42i32)]
        pub test_int: i32,

        #[field(env = "TEST_WRONG_TYPE", doc = "Test value", default = 42i32)]
        pub test_wrong_type: i32,

        #[field(env = "ERROR_TEST_BOOL", doc = "Test value", default = true)]
        pub test_bool: bool,
        #[field(env = "TEST_OPTIONAL", doc = "Test value", example = 123, optional)]
        pub test_optional: Option<i32>,
    }
);
fn main() {
    dotenvy::from_filename("./test.env").ok();
    match std::env::args().nth(1) {
        Some(arg) => match arg.as_str() {
            "default" => test_with_config(),
            "error" => test_with_config_error(),
            "error_result" => test_with_config_error_result(),
            "docs" => generate_docs(),
            "metadata" => show_metadata(),
            _ => println!(
                "unknown arg: {}. Available: default, error_result, docs, metadata",
                arg
            ),
        },
        None => {
            println!("Usage: util-cli [command]");
            println!("Commands:");
            println!("  default  - Test loading config with defaults");
            println!("  error    - Test loading config with errors");
            println!("  error_result - Test loading config with errors and Result");
            println!("  docs     - Generate CONFIG.md documentation");
            println!("  metadata - Show configuration metadata");
        }
    };
}

fn test_with_config() {
    let config = WorkingConfig::load();
    println!("Config loaded successfully!");
    println!("  test_string: {}", config.test_string);
    println!("  test_int: {}", config.test_int);
    println!("  test_bool: {}", config.test_bool);
}

fn test_with_config_error() {
    let _config = ErrorConfig::load();
    println!("you should not see this");
}

fn test_with_config_error_result() {
    let config = ErrorConfig::new();
    match config {
        Ok(config) => {
            println!("Config loaded successfully!");
            println!("  test_string: {}", config.test_string);
            println!("  test_int: {}", config.test_int);
            println!("  test_bool: {}", config.test_bool);
        }
        Err(errors) => {
            eprintln!("Failed to load config:");
            for error in errors {
                eprintln!("\t- {}", error);
            }
        }
    }
    println!("all done");
}

fn generate_docs() {
    println!("Generating documentation for WorkingConfig...");
    let builder = WorkingConfig::builder_for_docs();
    match builder.write_docs("CONFIG.md") {
        Ok(_) => println!("✓ Documentation written to CONFIG.md"),
        Err(e) => eprintln!("✗ Failed to write documentation: {}", e),
    }

    println!("\nGenerating documentation for ErrorConfig...");
    let builder = ErrorConfig::builder_for_docs();
    match builder.write_docs("ERROR_CONFIG.md") {
        Ok(_) => println!("✓ Documentation written to ERROR_CONFIG.md"),
        Err(e) => eprintln!("✗ Failed to write documentation: {}", e),
    }
}

fn show_metadata() {
    println!("WorkingConfig metadata:");
    let meta = WorkingConfig::metadata();
    println!("  test_string:");
    println!("    env: {}", meta.test_string.key);
    println!("    description: {}", meta.test_string.description);
    println!("    required: {}", meta.test_string.required);
    println!("  test_int:");
    println!("    env: {}", meta.test_int.key);
    println!("    description: {}", meta.test_int.description);
    println!("    default: {}", meta.test_int.default);
    println!("  test_bool:");
    println!("    env: {}", meta.test_bool.key);
    println!("    description: {}", meta.test_bool.description);
    println!("    default: {}", meta.test_bool.default);
}
