use poem::{EndpointExt, Route, endpoint::BoxEndpoint, get};

mod callback;
mod jwt;
mod login;

pub fn routes() -> BoxEndpoint<'static> {
    Route::new()
        .at("/login/:forge_id", get(login::login))
        .at("/callback", get(callback::callback))
        .boxed()
}
