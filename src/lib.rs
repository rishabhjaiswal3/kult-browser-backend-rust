pub mod config;
pub mod content;
pub mod game;
pub mod handler;
pub mod leaderboard;
pub mod logging;
pub mod middleware;
pub mod mongo;
pub mod player;
pub mod server;

// Re-export at crate root
pub use game::GameModel;
pub use game::GameModelRepository;
pub use middleware::{AuthPlayer, AuthService};
pub use player::{PlayerRepository, PlayerService};
