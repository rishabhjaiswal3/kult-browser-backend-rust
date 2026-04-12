pub mod connection;
pub mod indexes;

// Re-export connect at module level for cleaner API: mongo::connect()
pub use connection::connect;
pub use indexes::ensure_indexes;
