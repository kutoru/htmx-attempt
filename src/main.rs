use anyhow::{Context, Result};
use askama::Template;
use axum::{http::StatusCode, response::{Html, IntoResponse, Response}, routing::get, Router};
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "htmx_attempt=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pwd = std::env::current_dir().unwrap();
    let port = 727;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    let router = Router::new().route("/", get(hello)).nest_service(
        "/assets",
        ServeDir::new(format!("{}/assets", pwd.to_str().unwrap())),
    );

    info!("Listening on port {}", port);

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .context("could not start the server")?;

    Ok(())
}

async fn hello() -> impl IntoResponse {
    let template = HelloTemplate {};
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate;

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template: {}", err),
            )
                .into_response(),
        }
    }
}
