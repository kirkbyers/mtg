use mtg::db::{init_conn, search_cards, Card};
use rusqlite::Connection;
use tower_http::services::ServeDir;
use std::sync::Arc;
use tokio::sync::Mutex;
use axum::{extract::{Query, State}, routing::get, Json, Router};
use serde::Deserialize;

// Wrapper for SQLite connection
struct DbConnection(Mutex<Connection>);

#[tokio::main]
async fn main() {
    let conn = init_conn().expect("Failed to init db");
    let db = Arc::new(DbConnection(Mutex::new(conn)));
    // Create a new router
    let app = Router::new()
        .route("/api/cards", get(get_cards))
        .nest_service("/", ServeDir::new("www"))
        .with_state(db);

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct CardQueryParams {
    #[serde(default = "default_page")]
    page: u32,
    #[serde(default = "default_limit")]
    limit: u32,
    #[serde(default = "default_search")]
    search: String,
}

fn default_page() -> u32 {
    1
}

fn default_limit() -> u32 {
    100
}

fn default_search() -> String {
    String::new()
}

async fn get_cards(State(db): State<Arc<DbConnection>>, params: Query<CardQueryParams>) -> Json<Vec<Card>> {
    let page = params.page;
    let limit = params.limit;
    let search = params.search.clone();

    let conn = db.0.lock().await;

    match search_cards(&conn, &search, page, limit) {
        Ok(cards) => {
            println!("Found cards: {:?}", cards);
            Json(cards)
        },
        Err(e) => {
            println!("Error finding cards: {:?}", e);
            // TODO: logger the error
            vec![].into()
        }
    }
}
