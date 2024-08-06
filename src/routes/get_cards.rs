use crate::db::{search_cards, Card, DbConnection};
use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct CardQueryParams {
    #[serde(default = "default_page")]
    page: u32,
    #[serde(default = "default_limit")]
    limit: u32,
    #[serde(default = "default_search")]
    search: String,
}

pub fn default_page() -> u32 {
    1
}

pub fn default_limit() -> u32 {
    25
}

pub fn default_search() -> String {
    String::new()
}

pub async fn get_cards(
    State(db): State<Arc<DbConnection>>,
    params: Query<CardQueryParams>,
) -> Json<Vec<Card>> {
    let page = params.page;
    let limit = params.limit;
    let search = params.search.clone();

    let conn = db.0.lock().await;

    match search_cards(&conn, &search, page, limit) {
        Ok(cards) => {
            println!("Found cards: {:?}", cards);
            Json(cards)
        }
        Err(e) => {
            println!("Error finding cards: {:?}", e);
            // TODO: logger the error
            vec![].into()
        }
    }
}
