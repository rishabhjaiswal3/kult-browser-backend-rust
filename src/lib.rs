pub mod config;
pub mod content;
pub mod external;
pub mod game;
pub mod handler;
pub mod leaderboard;
pub mod logging;
pub mod middleware;
pub mod moments;
pub mod mongo;
pub mod player;
pub mod redis;
pub mod server;
pub mod upload;

// Re-export at crate root
pub use game::GameModel;
pub use game::GameModelRepository;
pub use middleware::{AuthPlayer, AuthService};
pub use player::{PlayerRepository, PlayerService};
