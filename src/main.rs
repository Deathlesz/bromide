use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

mod router;
mod schema;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "bromide=debug,tower_http=debug,axum::rejection=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("connection to the database");
    sqlx::any::install_default_drivers();

    let database_url = std::env::vars()
        .find(|(name, _)| name == "DATABASE_URL")
        .map(|(_, value)| value)
        .unwrap_or("sqlite:bromide.db".into());

    let pool = sqlx::AnyPool::connect(&database_url)
        .await
        .unwrap_or_else(|err| {
            error!("failed to connect to database `{database_url}`: {err}");
            std::process::exit(1);
        });

    let address = std::env::vars()
        .find(|(name, _)| name == "BIND_ADDRESS")
        .map(|(_, value)| value)
        .unwrap_or("0.0.0.0:8000".into());

    info!("trying to bind to `{address}`...");
    axum::serve(
        tokio::net::TcpListener::bind(address)
            .await
            .unwrap_or_else(|err| {
                error!("failed to bind: {err}");
                std::process::exit(2);
            }),
        router::router(pool),
    )
    .await
    .unwrap_or_else(|err| {
        error!("failed to start the server: {err}");
        std::process::exit(3);
    })
}
