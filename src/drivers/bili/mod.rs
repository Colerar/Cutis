use std::fmt::Debug;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::ops::Add;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::{fs, io};

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use bytes::Bytes;
use cookie::time::OffsetDateTime;
use cookie_store::{Cookie, CookieDomain, CookieExpiration, CookieStore};
use log::error;
use regex::Regex;
use reqwest::header::{ORIGIN, REFERER};
use reqwest::multipart::{Form, Part};
use reqwest::{Client, ClientBuilder, Url};
use reqwest_cookie_store::CookieStoreRwLock;
use tokio::time;
use tokio::time::timeout;
use tracing::{debug, info, warn};

use super::bili::url::ALBUM_UPLOAD_URL;
use super::Driver;

use self::data::{AlbumUploadRsp, LoginQrRsp, QrRsp, SelfInfoRsp};
use self::url::{BASIC_INFO_GET_URL, FEED_DOMAIN, LOGIN_QRCODE_GET_WEB_URL, LOGIN_WEB_QRCODE_URL};

pub mod data;
mod url;

pub(self) const MAC_SAFARI_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 12_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.6 Safari/605.1.15";

pub struct BiliClient {
  pub reqwest: Client,
  cookie_path: PathBuf,
  pub cookie: Arc<CookieStoreRwLock>,
}

impl BiliClient {
  fn reqwest(&self) -> Client {
    self.reqwest.clone()
  }

  pub async fn new() -> Result<BiliClient, anyhow::Error> {
    BiliClient::new_with_options(|i| i).await
  }

  pub async fn new_with_options<F>(option: F) -> Result<BiliClient, anyhow::Error>
  where
    F: FnOnce(ClientBuilder) -> ClientBuilder + Send,
  {
    let mut path = crate::dirs::DATA.clone();
    path.push("./bili_cookies.jsonl");
    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent).expect("Failed to create dir");
    }
    let file = OpenOptions::new()
      .create(true)
      .read(true)
      .write(true)
      .open(&path)
      .unwrap();
    let cookie_store = {
      let file = BufReader::new(&file);
      CookieStore::load_json(file).unwrap()
    };
    let cookie_store = CookieStoreRwLock::new(cookie_store);
    let cookie_store = Arc::new(cookie_store);

    let builder = Client::builder()
      .pool_max_idle_per_host(0)
      .cookie_provider(Arc::clone(&cookie_store))
      .user_agent(MAC_SAFARI_USER_AGENT);
    let client = option(builder)
      .build()
      .expect("Failed to create reqwest client");

    Ok(BiliClient {
      reqwest: client,
      cookie_path: path,
      cookie: Arc::clone(&cookie_store),
    })
  }

  async fn get_csrf(&self) -> Result<String, GetCsrfError> {
    let arc = Arc::clone(&self.cookie);
    let lock = &arc.read().map_err(|_| GetCsrfError::LockPoison())?;
    let cookie = lock.get("bilibili.com", "/", "bili_jct");
    if let Some(csrf) = cookie.map(|i| String::from(i.value())) {
      Ok(csrf)
    } else {
      Err(GetCsrfError::NotLogin())
    }
  }

  async fn save_cookies(&self) -> Result<(), SaveCookieError> {
    let cookie = self
      .cookie
      .write()
      .map_err(|_| SaveCookieError::LockPoison())?;
    let mut w = File::create(&self.cookie_path).map(BufWriter::new)?;
    cookie.save_json(&mut w)?;
    Ok(())
  }

  // region ======= Login =======

  async fn get_login_qr(&self) -> Result<QrRsp, reqwest::Error> {
    let result = self.reqwest().get(LOGIN_QRCODE_GET_WEB_URL).send().await?;
    let qr_data: QrRsp = result.json().await?;
    Ok(qr_data)
  }

  async fn login_qrcode(&self, qr_rsp: &QrRsp) -> Result<LoginQrRsp, LoginQrError> {
    let key = if let Some(key) = qr_rsp.data.as_ref().and_then(|x| x.oauth_key.as_ref()) {
      key
    } else {
      Err(LoginQrError::NoneOauthKey)?
    };
    let result = self
      .reqwest()
      .post(LOGIN_WEB_QRCODE_URL)
      .form(&[("oauthKey", key)])
      .send()
      .await
      .map_err(LoginQrError::Network)?;
    let login_rsp: LoginQrRsp = result.json().await.map_err(LoginQrError::Network)?;
    Ok(login_rsp)
  }

  // endregion ======= Login ======= //

  // region ======= Info =======

  async fn get_self_info(&self) -> Result<SelfInfoRsp, reqwest::Error> {
    let builder = self.reqwest().get(BASIC_INFO_GET_URL);
    let rsp = builder.send().await?;
    let info: SelfInfoRsp = rsp.json().await?;
    Ok(info)
  }

  // endregion ======= Info ======= //

  // region ======= Upload =======

  async fn upload_image_via_album(
    &self,
    init_part: Part,
  ) -> Result<AlbumUploadRsp, AlbumUploadError> {
    let rsp = self
      .reqwest()
      .post(ALBUM_UPLOAD_URL)
      .header(REFERER, FEED_DOMAIN)
      .header(ORIGIN, FEED_DOMAIN)
      .multipart(
        Form::new()
          .part(
            "file_up",
            init_part
              .mime_str("image/*")
              .unwrap()
              .file_name("B站未来有可能会倒闭，但绝不会变质"),
          )
          .text("biz", "draw")
          .text("category", "daily")
          .text("csrf", self.get_csrf().await?),
      )
      .send()
      .await?;
    let rsp: AlbumUploadRsp = rsp.json().await?;
    Ok(rsp)
  }

  // endregion ======= Upload ======= //
}

#[async_trait]
impl Driver for BiliClient {
  fn upload_need_login(&self) -> bool {
    true
  }

  fn download_need_login(&self) -> bool {
    false
  }

  async fn is_login(&self) -> Result<bool, anyhow::Error> {
    self
      .get_self_info()
      .await
      .map(|a| a.data.is_login)
      .context("Failed to get is login")
  }

  async fn print_self_info(&self) {
    match self.get_self_info().await {
      Ok(info) => {
        if info.data.is_login {
          info!("Login successfully!");
          info!(
            "User: {name}({uid})",
            name = info.data.username.unwrap_or_else(|| "Unknown".to_string()),
            uid = info
              .data
              .mid
              .map(|i| i.to_string())
              .unwrap_or_else(|| "Unknown".to_string()),
          );
        } else {
          error!("Login failed.")
        }
      }
      Err(err) => {
        error!("Failed to get self info: {err:?}")
      }
    }
  }

  async fn log_out(&self) -> Result<(), anyhow::Error> {
    {
      let mut cookie = self
        .cookie
        .write()
        .map_err(|_| anyhow::Error::msg("Poisoned RwLock"))?;
      cookie.clear();
    }
    self.save_cookies().await?;
    Ok(())
  }

  async fn qr_login(&self) -> Result<(), anyhow::Error> {
    let qr = self
      .get_login_qr()
      .await
      .context("Failed to get loginqr, network error")?;
    let url = qr
      .data
      .as_ref()
      .and_then(|qr| qr.url.clone())
      .context("Failed to get login qr, url is none")?;
    info!("Please open bilibili app, scan the qrcode below and confirm.");
    if qr2term::print_qr(url).is_err() {
      warn!("Failed to send qrcode via terminal");
      warn!("Please visit the website, then open bilibili app, scan the qrcode and confirm.");
      info!("")
    }
    let rsp: Result<(), _> = timeout(Duration::from_secs(120), async {
      loop {
        debug!("Try to login in");
        let login = self.login_qrcode(&qr).await;
        let success = match login {
          Ok(login) => login.code == 0,
          _ => false,
        };
        if success {
          break;
        }
        time::sleep(Duration::from_secs(5)).await;
      }
    })
    .await;
    if rsp.is_err() {
      return rsp.context("Long time (120 seconds) not login, timed out.");
    };
    tokio::join!(
      async {
        if let Err(err) = self.save_cookies().await {
          error!("Failed to save cookies: {err:?}");
        }
      },
      self.print_self_info(),
    );
    Ok(())
  }

  async fn cookie_login(&self, cookie: &str) -> Result<(), anyhow::Error> {
    {
      let mut store = self
        .cookie
        .write()
        .map_err(|_| anyhow!("Poisoned RwLock"))?;
      let url = "https://bilibili.com"
        .parse()
        .context("Failed to parse cookie domain")?;
      let raw_cookie = String::from(cookie);

      fn cookie_from_header<'a>(raw_cookie: &str, url: &Url) -> Result<Vec<Cookie<'a>>> {
        let mut cookies = Vec::new();
        let string = raw_cookie.replace(' ', "");
        let vec = string.split(';');
        for x in vec {
          let cookie = Cookie::parse(String::from(x), url).context("Failed to parse cookie")?;
          cookies.push(cookie);
        }
        Ok(cookies)
      }

      let mut cookies = cookie_from_header(&*raw_cookie, &url).context("Unable to parse cookie")?;
      store.clear();
      for x in cookies.iter_mut() {
        let time = OffsetDateTime::now_utc().add(Duration::new(60 * 24 * 60 * 365, 0));
        x.expires = CookieExpiration::AtUtc(time);
        x.domain = CookieDomain::Suffix("bilibili.com".to_string());
        store
          .insert(x.to_owned(), &url)
          .with_context(|| format!("Failed to insert cookie {:?}", x))?;
      }
    }
    {
      self.save_cookies().await?;
    }
    Ok(())
  }

  async fn upload_image(&self, data: Bytes) -> Result<Url, anyhow::Error> {
    debug!("Uploading image, size {}...", data.len());
    let rsp = self.upload_image_via_album(Part::stream(data)).await?;
    if rsp.code != 0 {
      return Err(anyhow!("Response json code != 0: {:#?}", rsp));
    }
    if let Some(data) = &rsp.data {
      if let Some(url) = &data.image_url {
        Ok(url.parse()?)
      } else {
        Err(anyhow!("Url is none {:#?}", &rsp))
      }
    } else {
      Err(anyhow!("Data is none {:#?}", &rsp))
    }
  }

  fn check_can_parse(&self, url: &str) -> bool {
    SHORT_FORM.is_match(url) || LONG_FORM.is_match(url)
  }

  fn abbr_url(&self, url: &str) -> Option<String> {
    if let Some(caps) = LONG_FORM.captures(url) {
      return Some(format!("bili://{}", &caps["hex"]));
    }
    None
  }

  fn un_abbr_url(&self, url: &str) -> Option<String> {
    if let Some(caps) = SHORT_FORM.captures(url) {
      return Some(format!(
        "https://i0.hdslb.com/bfs/album/{}.png",
        &caps["hex"]
      ));
    }
    None
  }
}

// regexes
lazy_static! {
  static ref SHORT_FORM: Regex = Regex::new(
    r#"(?x)
    ^
    bili://
    (?P<hex>
      [a-f0-9]{40}
    )
    $
    "#
  )
  .unwrap();
  static ref LONG_FORM: Regex = Regex::new(
    r#"(?x)
    ^
    https?://i0\.hdslb\.com/bfs/album/
    (?P<hex>
      [a-f0-9]{40}
    )
    \.
    (?P<format>
      png|jpeg
    )
    $
    "#
  )
  .unwrap();
}

#[derive(Debug, thiserror::Error)]
#[error("A error occurred during saving cookie")]
pub enum SaveCookieError {
  #[error("An I/O Error occurred")]
  Io(#[from] io::Error),
  #[error("Mutex lock poisoned")]
  LockPoison(),
  #[error("A CookieStore Error occurred")]
  CookieStore(#[from] cookie_store::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum LoginQrError {
  #[error("The oauth_key of QrRsp is None")]
  NoneOauthKey,
  #[error("A network error occurred {0}")]
  Network(#[from] reqwest::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum GetCsrfError {
  #[error("Mutex lock poisoned")]
  LockPoison(),
  #[error("Not login, no bili_jct value")]
  NotLogin(),
}

#[derive(Debug, thiserror::Error)]
pub enum AlbumUploadError {
  #[error("Invalid csrf")]
  Csrf(#[from] GetCsrfError),
  #[error("A network error occurred {0}")]
  Network(#[from] reqwest::Error),
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use reqwest::multipart::Part;
  use tracing::info;

  use crate::encoder::Encoder;
  use crate::init_logger;

  use super::BiliClient;
  use super::Driver;
  use super::GetCsrfError;

  lazy_static! {
    static ref TEST_CLI: Arc<BiliClient> = {
      async fn init() -> Arc<BiliClient> {
        Arc::new(BiliClient::new().await.unwrap())
      }
      futures::executor::block_on(init())
    };
  }

  #[tokio::test]
  async fn get_csrf_test() {
    match TEST_CLI.get_csrf().await {
      Ok(cookie) => {
        dbg!(cookie);
      }
      Err(err) => match err {
        GetCsrfError::NotLogin() => println!("Not login!"),
        _ => Err(err).unwrap(),
      },
    };
  }

  #[tokio::test]
  async fn get_login_qr_test() {
    dbg!(TEST_CLI.get_login_qr().await.unwrap());
  }

  #[tokio::test]
  async fn qr_code_login_test() {
    if option_env!("RUN_MANUALLY") != Some("true") {
      return;
    }
    init_logger(None);
    TEST_CLI.qr_login().await.unwrap();
    info!("Login successfully!");
  }

  #[tokio::test]
  async fn get_self_info_test() {
    let rsp = TEST_CLI.get_self_info().await.unwrap();
    dbg!(rsp);
  }

  #[tokio::test]
  async fn upload_image_via_album_test() {
    let enc = crate::encoder::png::PngEncoder();
    let encoded = enc.encode(&[80; 114514]).await.unwrap();
    let rsp = TEST_CLI
      .upload_image_via_album(Part::bytes(encoded))
      .await
      .unwrap();
    dbg!(rsp);
  }

  #[tokio::test]
  async fn bili_cookie_login() {
    if option_env!("RUN_MANUALLY") != Some("true") {
      return;
    }
    let cookie = std::env::var("BILI_COOKIE").unwrap();
    TEST_CLI.cookie_login(&*cookie).await.unwrap();
    dbg!(TEST_CLI.get_self_info().await.unwrap());
  }

  #[test]
  fn check_can_parse_test() {
    assert!(TEST_CLI.check_can_parse("bili://2569aaaa4f9b28787cf1f0c5b1134cc7e0900000"));
    assert!(TEST_CLI.check_can_parse(
      "http://i0.hdslb.com/bfs/album/2569aaaa4f9b28787cf1f0c5b1134cc7e0900000.png"
    ));
    assert!(TEST_CLI.check_can_parse(
      "https://i0.hdslb.com/bfs/album/2569aaaa4f9b28787cf1f0c5b1134cc7e0900000.png"
    ));
    assert!(!TEST_CLI.check_can_parse(
      "https://i0.hdslb.com/bfs/album/2569aaaa4f9b28787cf1f0c5b1134cc7e0900000.webm"
    ));
    assert!(!TEST_CLI.check_can_parse(
      "https://i0.hdslb.com/bfs/album/2569aa4f9b28787cf1f0c5b1134cc7e0900000.webm"
    ));
    assert!(!TEST_CLI.check_can_parse("bili://FAILED"));
    assert!(!TEST_CLI.check_can_parse("bili://2569aaaa4f9b28787cf1f0c5b1134cc7e0900000.png"));
    assert!(!TEST_CLI.check_can_parse("2569AAaa4f9b28787cf1f0c5b1134cc7e0900000"));
  }

  #[test]
  fn abbr_unabbr_url_test() {
    assert_eq!(
      TEST_CLI.un_abbr_url("bili://2569aaaa4f9b28787cf1f0c5b1134cc7e0900000"),
      Some(
        "https://i0.hdslb.com/bfs/album/2569aaaa4f9b28787cf1f0c5b1134cc7e0900000.png".to_string()
      ),
    );
    assert_eq!(
      TEST_CLI.un_abbr_url("bili://2569aaaa4f9b28787cf1f0c5b1134cc7e090"),
      None,
    );
    assert_eq!(
      TEST_CLI
        .abbr_url("https://i0.hdslb.com/bfs/album/2569aaaa4f9b28787cf1f0c5b1134cc7e0900000.png"),
      Some("bili://2569aaaa4f9b28787cf1f0c5b1134cc7e0900000".to_string()),
    );
  }
}
