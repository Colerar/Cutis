use std::sync::Arc;

use async_trait::async_trait;
use reqwest::{ClientBuilder, Url};

pub mod bili;

#[async_trait]
pub trait Driver {
  async fn new() -> anyhow::Result<Arc<Self>>;
  async fn new_with_options<F>(option: F) -> anyhow::Result<Arc<Self>>
  where
    F: FnOnce(ClientBuilder) -> ClientBuilder + Send;
  async fn is_login(&self) -> anyhow::Result<bool>;
  async fn log_out(&self) -> anyhow::Result<()>;
  async fn qr_login(&self) -> anyhow::Result<()>;
  async fn cookie_login(&self, cookie: &str) -> anyhow::Result<()>;

  async fn upload_image(&self, data: Vec<u8>) -> anyhow::Result<Url>;
  async fn download_image(&self, url: Url) -> anyhow::Result<Vec<u8>> {
    let bytes = reqwest::get(url).await?.bytes().await?;
    anyhow::Ok(Vec::from(bytes))
  }
}
