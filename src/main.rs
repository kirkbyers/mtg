use axum::{routing::get, Router};
use mtg::{
    db::{init_conn, DbConnection},
    routes::{get_card_vec_info, get_cards, get_vector_version},
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let conn = init_conn().expect("Failed to init db");
    let db = Arc::new(DbConnection(Mutex::new(conn)));
    // Create a new router
    let app = Router::new()
        .route("/api/cards", get(get_cards))
        .route("/api/vec_version", get(get_vector_version))
        .route("/api/card_vec_info", get(get_card_vec_info))
        .nest_service("/", ServeDir::new("www"))
        .with_state(db);

    // Start the server
    println!("Server starting on port 3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
