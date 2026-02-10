// Rust doesn't allow module names starting with a digit,
// so we use `zg` as the module name and re-export.
pub mod digital_ocean;
#[path = "0g/mod.rs"]
pub mod zg;

pub use digital_ocean::spaces;
pub use zg::storage;
