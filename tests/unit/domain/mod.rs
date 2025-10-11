// Domain layer unit tests
// These tests verify the business logic of core domain entities
// following Test-Driven Development (TDD) principles.

mod window_test;
mod workspace_test; 
mod monitor_test;

// Re-export test utilities for integration tests
pub use window_test::*;
pub use workspace_test::*; 
pub use monitor_test::*;