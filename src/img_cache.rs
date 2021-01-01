use blake3;
use reqwest::Url;
use std::{path::PathBuf};

use tree_magic_mini::{self, match_u8};
#[derive(Debug, Clone)]
pub struct ImageCache {
    path: PathBuf,
}

#[derive(Debug)]
pub struct RowImg<'a> {
    pub hash: String,
    pub filename: String,
    pub link: &'a Url,
}

impl ImageCache {
    pub async fn new(dir: &str) -> Result<ImageCache, std::io::Error> {
        tokio::fs::create_dir_all(&dir).await?;

        Ok(ImageCache {
            path: PathBuf::from(dir),
        })
    }
    pub async fn download(mut self, url: &'_ Url) -> Result<RowImg<'_>, anyhow::Error> {
        let bytes = reqwest::get(url.clone()).await?.bytes().await?;
        let extension = extension_from_guessed_mime(&bytes)?;
        let hash = self.hash_and_set_path(&bytes, extension);
        log::info!("cached img: {:#?}", &self.path);
        match tokio::fs::metadata(&self.path).await {
            Ok(_) => Ok(()),
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    tokio::fs::write(&self.path, bytes).await?;
                    Ok(())
                } else {
                    Err(err)
                }
            }
        }
        .map_err(|e| e)?;
        Ok(RowImg {
            link: url,
            filename: format!("{}.{}", &hash, extension),
            hash,
        })
    }

    fn hash_and_set_path(&mut self, bytes: &[u8], extension: &str) -> String {
        let hash = blake3::hash(&bytes).to_hex().to_string();
        self.path = self.path.join(&hash);
        self.path.set_extension(extension);
        hash
    }
}

fn extension_from_guessed_mime(bytes: &[u8]) -> Result<&'static str, anyhow::Error> {
    if match_u8(mime::IMAGE_JPEG.as_ref(), bytes) {
        Ok("jpeg")
    } else if match_u8(mime::IMAGE_PNG.as_ref(), bytes) {
        Ok("png")
    } else {
        Err(anyhow::format_err!("Expected jpeg or png as MIME Type."))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[tokio::test]
    async fn test_download_file() {
        let img_cache = ImageCache::new("img-cache").await.unwrap();
        let url_jpeg =
            Url::parse("http://www.bitsundso.de/wp-content/uploads/2012/05/bitsundso1400.jpg")
                .unwrap();
        log::info!("{:?}", img_cache.download(&url_jpeg,).await.unwrap());
        assert!(
            img_cache.download(&url_jpeg,).await.is_ok(),
            "MIME type is not image/jpeg"
        );

        let url_png = Url::parse("https://upload.wikimedia.org/wikipedia/commons/thumb/6/6a/PNG_Test.png/800px-PNG_Test.png").unwrap();
        assert!(
            img_cache.download(&url_png).await.is_ok(),
            "MIME type is not image/png"
        );
    }
}
