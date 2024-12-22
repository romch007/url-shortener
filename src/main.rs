mod endpoints;
mod errors;
mod id;

use std::{net::SocketAddr, time::Duration};

use axum::{
    routing::{get, post},
    Router,
};

use bb8_redis::RedisConnectionManager;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Clone)]
struct AppState {
    redis: bb8::Pool<RedisConnectionManager>,
}

async fn create_app_state() -> AppState {
    let redis_url = std::env::var("REDIS_URL").expect("no REDIS_URL env variable");
    let redis_manager =
        RedisConnectionManager::new(redis_url).expect("cannot create Redis connection manager");

    let pool = bb8::Pool::builder()
        .connection_timeout(Duration::from_secs(5))
        .build(redis_manager)
        .await
        .expect("cannot create Redis connection pool");

    AppState { redis: pool }
}

fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(endpoints::index))
        .route("/health", get(endpoints::health))
        .route("/:link", get(endpoints::get_link))
        .route("/", post(endpoints::set_link))
}

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app_state = create_app_state().await;
    let app = router().with_state(app_state);

    let addr = std::env::var("HOST").unwrap_or_else(|_| String::from("127.0.0.1"));
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| String::from("3000"))
        .parse::<u16>()
        .expect("invalid PORT");

    let addr = format!("{addr}:{port}")
        .parse::<SocketAddr>()
        .expect("invalid socket addr");

    tracing::info!("listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
