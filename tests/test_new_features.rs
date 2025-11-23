use config_loadr::{Load, define_config};

// Test: Allow missing docs
define_config! {
    #[allow(missing_docs)]
    pub struct ConfigWithoutDocs {
        #[field(env = "TEST_NO_DOCS_PORT", default = 8080u16)]
        pub port: u16,

        #[field(env = "TEST_NO_DOCS_HOST", doc = "Host can still have docs", default = String::from("localhost"))]
        pub host: String,
    }
}

#[test]
fn test_allow_missing_docs() {
    // This should compile and run without requiring #[doc] on all fields
    let config = ConfigWithoutDocs::load();
    assert_eq!(*config.port, 8080);
    assert_eq!(*config.host, "localhost");
}
