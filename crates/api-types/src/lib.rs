use poem_openapi::Object;

mod agent_mtmt;
pub use agent_mtmt::*;

mod agent;
pub use agent::*;

mod forge;
pub use forge::*;

mod user;
pub use user::*;

mod team;
pub use team::*;

mod repo;
pub use repo::*;

mod pipeline;
pub use pipeline::*;

#[derive(Debug, Object)]
pub struct ErrorResponse {
    pub message: String,
}
