use anyhow::Result;
use fastembed::{InitOptions, TextEmbedding};

pub fn init() -> Result<TextEmbedding> {
    let model = TextEmbedding::try_new(InitOptions {
        show_download_progress: true,
        ..Default::default()
    })?;

    Ok(model)
}

pub fn string_to_embedding(inp: &str, model: &TextEmbedding) -> Result<Vec<f32>> {
    let res = model.embed(vec![inp], None)?;

    Ok(res[0].to_owned())
}
