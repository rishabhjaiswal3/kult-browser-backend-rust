pub mod connection;

// Re-export connect at module level for cleaner API: mongo::connect()
pub use connection::connect;
