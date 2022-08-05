use async_trait::async_trait;
use reqwest::Url;

pub mod bili;

#[async_trait]
pub trait Driver: Send + Sync {
  async fn is_login(&self) -> anyhow::Result<bool>;
  async fn print_self_info(&self);
  async fn log_out(&self) -> anyhow::Result<()>;
  async fn qr_login(&self) -> anyhow::Result<()>;
  async fn cookie_login(&self, cookie: &str) -> anyhow::Result<()>;

  async fn upload_image(&self, data: Vec<u8>) -> anyhow::Result<Url>;
  // async fn download_image(&self, url: Url) -> anyhow::Result<Vec<u8>> {
  //   let bytes = reqwest::get(url).await?.bytes().await?;
  //   anyhow::Ok(Vec::from(bytes))
  // }

  /// Check a [url] can or not be parsed by this [Driver]
  fn check_can_parse(&self, _url: &str) -> bool {
    false
  }

  /// Abbreviate a [url] to short form
  ///
  /// # Return
  ///
  /// Return [None] if no short form available
  fn abbr_url(&self, _url: &str) -> Option<String> {
    None
  }

  /// Convert a Url from abbreviation to normal form
  ///
  /// ### Return
  ///
  /// Return [None] if input [url] is not a short form
  fn un_abbr_url(&self, _url: &str) -> Option<String> {
    None
  }
}
