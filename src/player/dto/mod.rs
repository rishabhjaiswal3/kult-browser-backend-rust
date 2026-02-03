mod login_dto;
mod profile_dto;
mod update_name_dto;

pub use login_dto::{LoginRequest, LoginResponse, PlayerInfo};
pub use profile_dto::{GameScoreEntry, PlayerProfile, PlayerProfileResponse};
pub use update_name_dto::{UpdateNameRequest, UpdateNameResponse};