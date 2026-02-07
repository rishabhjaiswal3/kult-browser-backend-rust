pub mod route;

pub mod controller;
pub mod model;
pub mod repository;
pub mod service;

// Re-export routes at module level
pub use route::routes;
