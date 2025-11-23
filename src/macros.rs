// This module provides helper functions used by the procedural macro
// The actual define_config! macro is in config-loadr-macros crate

/// Helper to validate a value at compile time
/// This is used by the generated code to check defaults
#[doc(hidden)]
pub const fn validate_const<T>(_value: &T) {}
