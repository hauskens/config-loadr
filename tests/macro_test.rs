use config_loadr::define_config;

define_config! {
    pub struct DefaultConfig {
        #[field(env = "TEST_DEFAULT_PORT", doc = "Server port", default = 8080u16)]
        pub port: u16,

        #[field(env = "TEST_DEFAULT_HOST", doc = "Server host", default = String::from("localhost"))]
        pub host: String,

        #[field(env = "TEST_DEFAULT_DEBUG", doc = "Enable debug mode", default = false)]
        pub debug: bool,

        #[field(env = "TEST_DEFAULT_NAME", doc = "Service name", default = String::from("test-service"))]
        pub name: String,

        #[field(env = "TEST_DEFAULT_OPTIONAL", doc = "Optional value", example = "test".into(), optional)]
        pub optional_value: Option<String>,
    }
}

define_config! {
    pub struct LoadRequiredFromEnvConfig {
        // Loaded from test.env, TEST_INT=42
        #[field(env = "TEST_INT", doc = "Integer value", required, example = 8080i32)]
        pub int: i32,

        // Loaded from test.env, TEST_STRING=test
        #[field(env = "TEST_STRING", doc = "String value", required, example = "Some different string".into())]
        pub string: String,

        // Loaded from test.env, TEST_BOOL_FALSE=false
        #[field(env = "TEST_BOOL_FALSE", doc = "Boolean value", required, example = true)]
        pub bool_false: bool,

        // Loaded from test.env, TEST_BOOL_TRUE=true
        #[field(env = "TEST_BOOL_TRUE", doc = "Boolean value", required, example = false)]
        pub bool_true: bool,
    }
}

define_config! {
    pub struct LoadOptionalFromEnvConfig {
        // Loaded from test.env, TEST_INT=42
        #[field(env = "TEST_INT", doc = "Integer value", optional, default = 8080i32)]
        pub int: i32,

        // Loaded from test.env, TEST_STRING=test
        #[field(env = "TEST_STRING", doc = "String value", optional, default = "localhost".into())]
        pub string: String,

        // Loaded from test.env, TEST_BOOL_FALSE=false
        #[field(env = "TEST_BOOL_FALSE", doc = "Boolean value", optional, default = false)]
        pub bool_false: bool,

        // Loaded from test.env, TEST_BOOL_TRUE=true
        #[field(env = "TEST_BOOL_TRUE", doc = "Boolean value", optional, default = true)]
        pub bool_true: bool,
    }
}

define_config! {
    pub struct MissingRequiredFromEnvConfig {
        // Loaded from test.env, TEST_INT=42
        #[field(env = "MISSING_TEST_INT", doc = "Integer value", required, example = 8080i32)]
        pub int: i32,

        // Loaded from test.env, TEST_STRING=test
        #[field(env = "MISSING_TEST_STRING", doc = "String value", required, example = "Some different string".into())]
        pub string: String,

        // Loaded from test.env, TEST_BOOL_FALSE=false
        #[field(env = "MISSING_TEST_BOOL_FALSE", doc = "Boolean value", required, example = true)]
        pub bool_false: bool,

        // Loaded from test.env, TEST_BOOL_TRUE=true
        #[field(env = "MISSING_TEST_BOOL_TRUE", doc = "Boolean value", required, example = false)]
        pub bool_true: bool,
    }
}

#[test]
fn test_macro_default_values() {
    let config = DefaultConfig::load();
    assert_eq!(config.port, 8080);
    assert_eq!(config.host, "localhost");
    assert!(!config.debug);
    assert_eq!(config.name, "test-service");
    assert_eq!(config.optional_value, None);
}

#[test]
fn test_macro_load_required_from_env() {
    // Should be able to load from environment
    dotenvy::from_filename("./test.env").ok();
    let config = LoadRequiredFromEnvConfig::load();
    assert_eq!(config.int, 42);
    assert_eq!(config.string, "test");
    assert!(!config.bool_false);
    assert!(config.bool_true);
}

#[test]
fn test_macro_load_optional_from_env() {
    // Loading from environment should override defaults
    dotenvy::from_filename("./test.env").ok();
    let config = LoadOptionalFromEnvConfig::load();
    assert_eq!(config.int, 42);
    assert_eq!(config.string, "test");
    assert!(!config.bool_false);
    assert!(config.bool_true);
}

#[test]
fn test_macro_missing_required_from_env() {
    // Should fail to load from environment
    dotenvy::from_filename("./test.env").ok();
    let config = MissingRequiredFromEnvConfig::new();
    assert!(config.is_err());
}
