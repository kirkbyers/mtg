use fastembed::{InitOptions, TextEmbedding};

pub fn init() -> Result<TextEmbedding, Box<dyn std::error::Error>> {
    let model = TextEmbedding::try_new(InitOptions {
        show_download_progress: true,
        ..Default::default()
    })?;

    Ok(model)
}

pub fn string_to_embedding(inp: &str, model: &TextEmbedding) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let res = model.embed(vec![inp], None)?;

    Ok(res[0].to_owned())
}   