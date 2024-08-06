use crate::{
    db::DbConnection,
    embedings::{init, string_to_embedding},
};
use axum::{
    extract::{
        rejection::{self, JsonRejection},
        State,
    },
    Json,
};
use reqwest::StatusCode;
use rusqlite::Row;
use std::sync::Arc;

pub async fn get_vector_version(State(db): State<Arc<DbConnection>>) -> (StatusCode, Json<String>) {
    let conn = db.0.lock().await;

    let vec_verstion: String = match conn.query_row("SELECT vec_version();", [], |row| row.get(0)) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(format!("Unable to get vec_version. \n\n {}", e)),
            )
        }
    };

    (
        StatusCode::OK,
        Json(format!("vec_version {}", vec_verstion)),
    )
}

pub async fn get_card_vec_info(State(db): State<Arc<DbConnection>>) -> (StatusCode, Json<String>) {
    let conn = db.0.lock().await;

    let model = match init() {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(format!("Failed to init fastembed model. \n\n {}", e)),
            )
        }
    };
    let search = match string_to_embedding("flying hexproof", &model) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(format!("Failed to embedd text. \n\n {}", e)),
            )
        }
    };

    let mut stmt = match conn.prepare(&format!(
        "
        SELECT cv.rowid, cv.distance, c.name, c.oracle_text
        FROM card_vecs as cv
        JOIN cards as c
        ON c.rowid = cv.rowid
        WHERE embedding match '{:?}'
        and k = 10
        ORDER BY distance
        LIMIT 10;
    ",
        search
    )) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(format!("Unable to prepare statement. \n\n {}", e)),
            )
        }
    };

    let rows = match stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    }) {
        Ok(rows) => rows,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(format!("Unable to get vec rows. \n\n {}", e)),
            )
        }
    };

    let mut row_ids: Vec<(i32, f64, String, String)> = Vec::new();
    for row in rows {
        match row {
            Ok(id) => row_ids.push(id),
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(format!("Error while iterating rows. \n\n {}", e)),
                )
            }
        }
    }

    if row_ids.is_empty() {
        return (StatusCode::NOT_FOUND, Json("No rows found.".to_string()));
    }

    let res: Vec<String> = row_ids.into_iter().map(|x| format!("{:?}", x)).collect();

    (StatusCode::OK, Json(format!("{:?}", res)))
}
