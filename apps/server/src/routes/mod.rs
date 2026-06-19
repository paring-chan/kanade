use std::{fs::File, io::Read, path::PathBuf};

use poem::{
    EndpointExt, IntoResponse, Route,
    endpoint::{BoxEndpoint, StaticFilesEndpoint},
    error::{ResponseError, StaticFileError},
    web::Html,
};
use poem_openapi::OpenApiService;

mod auth;
mod sse;
mod webhook;
mod ws;

static FRONTEND_PATH: Option<&str> = option_env!("FRONTEND_PATH");

pub fn routes() -> BoxEndpoint<'static> {
    let svc = OpenApiService::new(crate::api::api(), "Kanade API", "1.0").url_prefix("/api/v1");

    let mut route = Route::new();

    if let Some(frontend_path) = FRONTEND_PATH {
        route = route.nest("/", StaticFilesEndpoint::new(frontend_path));
    }

    let route = route
        .nest("/_/webhook", webhook::routes())
        .nest("/_/auth", auth::routes())
        .nest("/api", svc.scalar())
        .nest("/api/v1", svc)
        .nest("/_/ws", ws::routes())
        .nest("/_/sse", sse::routes());

    if let Some(frontend_path) = FRONTEND_PATH {
        let mut index = String::new();
        File::open(PathBuf::from(frontend_path).join("index.html"))
            .unwrap()
            .read_to_string(&mut index)
            .unwrap();

        route
            .catch_error(move |e: StaticFileError| {
                let index = index.clone();
                async move {
                    match e {
                        StaticFileError::NotFound => Html(index.clone()).into_response(),
                        e => e.as_response(),
                    }
                }
            })
            .boxed()
    } else {
        route.boxed()
    }
}
