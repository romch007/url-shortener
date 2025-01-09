use crate::{errors::InternalErrExt, id, AppState};

use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::Html,
};

use bb8_redis::redis::{cmd, AsyncCommands};

pub async fn index() -> Html<&'static str> {
    Html(include_str!("index.html"))
}

pub async fn health(State(app_state): State<AppState>) -> Result<(), StatusCode> {
    let mut conn = app_state.redis.get().await.map_internal_err()?;

    let reply: String = cmd("PING")
        .query_async(&mut *conn)
        .await
        .map_internal_err()?;

    if reply != "PONG" {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }

    Ok(())
}

pub async fn get_link(
    Path(id): Path<String>,
    State(app_state): State<AppState>,
) -> Result<(StatusCode, HeaderMap), StatusCode> {
    let mut conn = app_state.redis.get().await.map_internal_err()?;

    let link: Option<String> = conn.get(&id).await.map_internal_err()?;

    if let Some(link) = link {
        let mut headers = HeaderMap::new();
        let header_value = HeaderValue::from_str(&link).map_internal_err()?;

        headers.insert(header::LOCATION, header_value);

        Ok((StatusCode::MOVED_PERMANENTLY, headers))
    } else {
        Ok((StatusCode::NOT_FOUND, HeaderMap::new()))
    }
}

pub async fn set_link(
    State(app_state): State<AppState>,
    body: String,
) -> Result<String, StatusCode> {
    let mut conn = app_state.redis.get().await.map_internal_err()?;
    let link_id = id::generate();

    let _: () = conn.set(&link_id, &body).await.map_internal_err()?;

    Ok(link_id)
}

#[cfg(test)]
mod tests {
    use crate::{create_app_state, id, router};
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
    };
    use bb8_redis::redis::AsyncCommands;
    use dotenvy::dotenv;
    use tower::ServiceExt;

    #[tokio::test]
    async fn health() {
        let _ = dotenv();
        let app_state = create_app_state().await;
        let app = router();
        let app = app.with_state(app_state);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn get_link() {
        let _ = dotenv();
        let app_state = create_app_state().await;
        let app = router();
        let app = app.with_state(app_state.clone());

        // Create a fake link entry
        let mut redis_conn = app_state.redis.get().await.unwrap();

        let fake_id = id::generate();

        let _: () = redis_conn
            .set(&fake_id, "https://google.com")
            .await
            .unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/{fake_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::MOVED_PERMANENTLY);
        assert_eq!(
            response.headers().get("Location").unwrap(),
            "https://google.com"
        );
    }

    #[tokio::test]
    async fn set_link() {
        let _ = dotenv();
        let app_state = create_app_state().await;
        let app = router();
        let app = app.with_state(app_state);

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/")
                    .body(Body::from("https://example.org"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
