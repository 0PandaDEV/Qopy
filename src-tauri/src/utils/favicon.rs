use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use image::ImageFormat;
use reqwest;
use url::Url;

pub async fn fetch_favicon_as_base64(
    url: Url,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let favicon_url = format!("https://favicone.com/{}", url.host_str().unwrap());
    let response = client.get(&favicon_url).send().await?;

    if response.status().is_success() {
        let bytes = response.bytes().await?;
        let img = image::load_from_memory(&bytes)?;
        let mut png_bytes: Vec<u8> = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut png_bytes), ImageFormat::Png)?;
        Ok(Some(STANDARD.encode(&png_bytes)))
    } else {
        Ok(None)
    }
}
