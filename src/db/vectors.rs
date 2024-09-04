use indicatif::{ProgressBar, ProgressStyle};
use rand::{seq::SliceRandom, thread_rng};
use rusqlite::{Connection, Result, Statement};

pub fn get_vec_version_stmt(conn: &Connection) -> Result<Statement> {
    conn.prepare("SELECT vec_version();")
}

pub fn prep_get_vec_count(conn: &Connection) -> Result<Statement> {
    conn.prepare("SELECT COUNT(*) FROM card_vecs;")
}

pub fn prep_get_all_embeddings(conn: &Connection) -> Result<Statement> {
    conn.prepare("SELECT embedding, rowid FROM card_vecs;")
}

pub const SELECT_PAGINATED_SEMANTIC_SEARCH: &str = "
    SELECT c.id, c.oracle_id, c.name, c.lang, 
        c.released_at, c.mana_cost, c.cmc, 
        c.type_line, c.oracle_text, c.power, 
        c.toughness, c.rarity, c.flavor_text, 
        c.artist, c.set_code, c.collector_number, 
        c.digital, iu.normal, cv.rowid, cv.distance
    FROM card_vecs as cv
    JOIN cards as c
    ON c.rowid = cv.rowid
    JOIN image_uris as iu
    ON c.id = iu.card_id
    WHERE embedding match :search
    and k = :limit
    GROUP BY c.name 
    ORDER BY distance
    LIMIT :limit
    OFFSET :offset;
    ";

/// Calculates the Euclidean distance between two float arrays.
///
/// # Arguments
///
/// * `a` - The first float array.
/// * `b` - The second float array.
///
/// # Returns
///
/// The calculated Euclidean distance as a float.
pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f32>()
        .sqrt()
}

#[derive(Debug, Clone)]
pub struct Point {
    pub rowid: f32,
    pub embedding: Vec<f32>,
}

pub fn k_means(points: &[Point], k: usize, max_iterations: usize) -> Vec<usize> {
    let n = points.len();
    let dim = points[0].embedding.len();

    // Initialize centroids randomly
    let mut centroids: Vec<Vec<f32>> = points
        .choose_multiple(&mut thread_rng(), k)
        .map(|p| p.embedding.clone())
        .collect();

    let mut assignments = vec![0; n];

    let progress_bar = ProgressBar::new(max_iterations as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} iterations (ETA: {eta})")
            .unwrap()
            .progress_chars("##-"),
    );

    for _ in 0..max_iterations {
        // Assign points to nearest centroid
        for (i, point) in points.iter().enumerate() {
            assignments[i] = (0..k)
                .min_by_key(|&j| {
                    let dist = euclidean_distance(&point.embedding, &centroids[j]);
                    (dist * 1000.0) as i32 // Convert to integer for comparison
                })
                .unwrap();
        }

        // Update centroids
        let mut new_centroids = vec![vec![0.0; dim]; k];
        let mut counts = vec![0; k];

        for (i, point) in points.iter().enumerate() {
            let cluster = assignments[i];
            for j in 0..dim {
                new_centroids[cluster][j] += point.embedding[j];
            }
            counts[cluster] += 1;
        }

        for i in 0..k {
            if counts[i] > 0 {
                for j in 0..dim {
                    new_centroids[i][j] /= counts[i] as f32;
                }
            }
        }

        // Check for convergence
        if centroids == new_centroids {
            progress_bar.finish_with_message("Converged early");
            break;
        }
        centroids = new_centroids;

        progress_bar.inc(1);
    }

    progress_bar.finish();
    assignments
}
