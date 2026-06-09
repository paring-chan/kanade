use poem::{EndpointExt, Route, endpoint::BoxEndpoint};
use poem_openapi::OpenApiService;

mod auth;
mod webhook;

pub fn routes() -> BoxEndpoint<'static> {
    let svc = OpenApiService::new(crate::api::api(), "Kanade API", "1.0").url_prefix("/api/v1");

    Route::new()
        .nest("/_/webhook", webhook::routes())
        .nest("/_/auth", auth::routes())
        .nest("/api", svc.scalar())
        .nest("/api/v1", svc)
        .boxed()
}
