use config_loadr::define_config;

define_config! {
    pub struct Config {
        #[field(env = "TEST_PORT", doc = "Server port", default = 8080u16)]
        pub port: u16,

        #[field(env = "TEST_HOST", doc = "Server host", default = String::from("localhost"))]
        pub host: String,

        #[field(env = "TEST_DEBUG", doc = "Debug mode", default = false)]
        pub debug: bool,
    }
}

#[test]
fn test_direct_access_no_deref() {
    let config = Config::load();

    // Direct access without dereferencing
    assert_eq!(config.port, 8080);
    assert_eq!(config.host, "localhost");
    assert!(!config.debug);

    // Can use in string formatting directly
    let message = format!("Server running on {}:{}", config.host, config.port);
    assert_eq!(message, "Server running on localhost:8080");
}

#[test]
fn test_metadata_access() {
    let metadata = Config::metadata();

    // Access metadata
    assert_eq!(metadata.port.key, "TEST_PORT");
    assert_eq!(metadata.port.description, "Server port");
    assert_eq!(metadata.port.default, 8080);
    assert!(!metadata.port.required);

    assert_eq!(metadata.host.key, "TEST_HOST");
    assert_eq!(metadata.host.description, "Server host");
    assert_eq!(metadata.host.default, "localhost");

    assert_eq!(metadata.debug.key, "TEST_DEBUG");
    assert_eq!(metadata.debug.description, "Debug mode");
    assert!(!metadata.debug.default);
}

#[test]
fn test_can_pass_to_functions() {
    let config = Config::load();

    fn takes_port(port: u16) -> u16 {
        port * 2
    }

    fn takes_host(host: &str) -> String {
        format!("https://{}", host)
    }

    // Can pass directly to functions without dereferencing
    assert_eq!(takes_port(config.port), 16160);
    assert_eq!(takes_host(&config.host), "https://localhost");
}
