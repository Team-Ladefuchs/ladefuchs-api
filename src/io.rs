use std::{
    io,
    path::{Path, PathBuf},
};

use axum::body::Body;
use axum::http::header;
use tokio_util::io::ReaderStream;

pub const BANNER_PATH: &str = "./images/banners";

pub async fn init_banner_folder() -> Result<(), io::Error> {
    let banner_folder = Path::new(BANNER_PATH);
    if !banner_folder.exists() {
        tokio::fs::create_dir_all(&banner_folder).await?;
    }
    Ok(())
}
fn remove_whitespace_filename(path: &Path) -> String {
    match path.file_name() {
        Some(path) => path
            .to_string_lossy()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<_>(),
        None => String::from("banner.jpg"),
    }
}

pub async fn read_file_stream<P: AsRef<Path>>(
    path: P,
) -> Result<(header::HeaderMap, Body), tokio::io::Error> {
    let path = path.as_ref();
    let file = tokio::fs::File::open(path).await?;
    let file_len = file.metadata().await?.len();
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        tree_magic_mini::from_filepath(path)
            .unwrap_or_default()
            .try_into()
            .map_err(|_e| {
                tokio::io::Error::new(tokio::io::ErrorKind::InvalidInput, "No mime type found")
            })?,
    );
    headers.insert(
        header::CONTENT_LENGTH,
        file_len.to_string().parse().unwrap(),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!(
            "attachment; filename=\"{}\"",
            remove_whitespace_filename(path)
        )
        .parse()
        .map_err(|_e| {
            tokio::io::Error::new(tokio::io::ErrorKind::InvalidData, "Invalid filename")
        })?,
    );
    Ok((headers, body))
}

pub async fn guess_image_mime<P: AsRef<Path>>(path: P) -> Result<mime::Mime, eyre::Error> {
    let path = path.as_ref();
    let mime_types = [
        mime::IMAGE_JPEG,
        mime::IMAGE_PNG,
        mime::IMAGE_SVG,
        mime::IMAGE_GIF,
    ];
    let bytes = read_bytes(path, 2048).await?;
    let guess_mime = tree_magic_mini::from_u8(&bytes);

    for valid_mime in mime_types {
        if guess_mime == valid_mime {
            return Ok(valid_mime);
        }
    }

    Err(eyre::Error::msg(format!(
        "Unsupported file type path: {}, type: {:#?}",
        path.to_string_lossy(),
        guess_mime
    )))
}

async fn read_bytes(filepath: &Path, byte_count: usize) -> Result<Vec<u8>, std::io::Error> {
    use tokio::fs::File;
    use tokio::io::AsyncReadExt;

    let mut bytes = Vec::<u8>::with_capacity(byte_count);
    let file = File::open(filepath).await?;
    file.take(byte_count as u64).read_to_end(&mut bytes).await?;
    Ok(bytes)
}

pub async fn hash_file(file: &PathBuf) -> Result<String, std::io::Error> {
    let bytes = tokio::fs::read(file).await?;
    let hash = blake3::hash(&bytes).to_hex().to_string();
    Ok(hash)
}
