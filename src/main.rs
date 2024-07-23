use axum::Router;
use reqwest::StatusCode;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    // Create a new router
    let app = Router::new().nest_service("/", ServeDir::new("www"));

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
