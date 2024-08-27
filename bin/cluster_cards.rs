use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use mtg::db::{
    init_conn, insert_cluster_assignments,
    vectors::{k_means, prep_get_all_embeddings, prep_get_vec_count, Point},
};

fn main() -> Result<()> {
    let conn = init_conn()?;
    let mut count_stmt = prep_get_vec_count(&conn)?;
    let count: i64 = count_stmt.query_row([], |row| row.get(0))?;

    let progress_bar = ProgressBar::new(count as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} records (ETA: {eta})")
            .unwrap()
            .progress_chars("##-"),
    );

    let mut get_embeds_stmt = prep_get_all_embeddings(&conn)?;
    let points: Vec<Point> = get_embeds_stmt
        .query_map([], |row| {
            let embedding: Vec<u8> = row.get(0)?;
            let embedding_f32: Vec<f32> = embedding
                .chunks_exact(4)
                .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
                .collect();
            progress_bar.inc(1);
            Ok(Point {
                embedding: embedding_f32,
                rowid: row.get(1)?,
            })
        })?
        .map(|res| res.map_err(anyhow::Error::from)) // Convert rusqlite::Error to anyhow::Error
        .collect::<Result<Vec<Point>>>()?;
    progress_bar.finish_with_message("Card embeddings loaded");

    // Perform k-means clustering
    let k = 30; // Number of clusters
    let max_iterations = 100;

    println!(
        "Starting k-means clustering with k={} and max_iterations={}",
        k, max_iterations
    );
    let assignments = k_means(&points, k, max_iterations);

    println!("Clustering completed. Saving assignments");

    insert_cluster_assignments(&conn, &assignments, &points)?;
    Ok(())
}
