use config_loadr::define_config;

define_config! {
    /// Test configuration
    pub struct TestConfig {
        #[field(env = "TEST_CFG_PORT", doc = "Server port", default = 8080u16)]
        pub port: u16,

        #[field(env = "TEST_CFG_HOST", doc = "Server host", default = String::from("localhost"))]
        pub host: String,

        #[field(env = "TEST_CFG_DEBUG", doc = "Enable debug mode", default = false)]
        pub debug: bool,

        #[field(env = "TEST_CFG_NAME", doc = "Service name", example = String::from("test-service"), required)]
        pub name: String,

        #[field(env = "TEST_CFG_OPTIONAL", doc = "Optional value", example = "example", optional)]
        pub optional_value: Option<String>,
    }
}
