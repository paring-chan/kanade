mod agent;
pub use agent::*;

mod forge;
pub use forge::*;

mod user;
use poem_openapi::Object;
pub use user::*;

mod team;
pub use team::*;

#[derive(Debug, Object)]
pub struct ErrorResponse {
    pub message: String,
}
