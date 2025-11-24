use config_loadr::define_config;

// For testing missing optional fields (no errors expected)
define_config! {
    #[derive(Debug)]
    pub struct MissingOptConfig {
        #[field(env = "TEST_MISSING_OPT_PORT", doc = "Optional port", example = 8080, optional)]
        pub port: Option<u16>,
    }
}

// For testing wrong type errors (errors expected)
define_config! {
    #[derive(Debug)]
    pub struct WrongTypeConfig {
        #[field(env = "TEST_WRONG_TYPE", doc = "Optional port with wrong type", example = 8080, optional)]
        pub wrong_type: Option<u16>,
    }
}

#[test]
fn test_optional_field_missing_returns_none() {
    dotenvy::from_filename("./test.env").ok();
    // TEST_MISSING_OPT_PORT should not be set
    let config = MissingOptConfig::load();

    assert_eq!(config.port, None);
}

#[test]
fn test_optional_field_wrong_type_returns_error() {
    dotenvy::from_filename("./test.env").ok();
    // TEST_WRONG_TYPE should be configured, but not be an int
    let config = WrongTypeConfig::new();

    assert!(config.is_err());
}

#[test]
#[should_panic]
fn test_optional_field_wrong_type_should_panic() {
    dotenvy::from_filename("./test.env").ok();
    // TEST_WRONG_TYPE should be configured, but not be an int
    let _config = WrongTypeConfig::load();
}
