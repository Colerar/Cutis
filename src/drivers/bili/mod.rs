use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::sync::Arc;
use std::{fs, io};

use cookie_store::CookieStore;
use rand::seq::SliceRandom;
use reqwest::header::{ORIGIN, REFERER};
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use reqwest_cookie_store::CookieStoreRwLock;

use self::data::AlbumUploadRsp;
use self::url::{ALBUM_UPLOAD_URL, FEED_DOMAIN};

use self::data::{LoginQrRsp, QrRsp, SelfInfoRsp};
use self::url::{BASIC_INFO_GET_URL, LOGIN_QRCODE_GET_WEB_URL, LOGIN_WEB_QRCODE_URL};

pub mod data;
mod url;

pub(self) const MAC_SAFARI_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 12_5) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.6 Safari/605.1.15";

pub struct BiliClient {
  pub reqwest: Client,
  cookie_path: PathBuf,
  pub cookie: Arc<CookieStoreRwLock>,
}

impl BiliClient {
  async fn new() -> BiliClient {
    let mut path = crate::dirs::DATA.clone();
    path.push("./cookies.jsonl");
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
    let client = builder.build().expect("Failed to create reqwest client");

    BiliClient {
      reqwest: client,
      cookie_path: path,
      cookie: Arc::clone(&cookie_store),
    }
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
    let result = self.reqwest.get(LOGIN_QRCODE_GET_WEB_URL).send().await?;
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
      .reqwest
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
    let rsp = self.reqwest.get(BASIC_INFO_GET_URL).send().await?;
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
      .reqwest
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
              .file_name(*QUOTES.choose(&mut rand::thread_rng()).unwrap()),
          )
          .text("biz", "new_dyn")
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

#[derive(Debug, thiserror::Error)]
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
  use reqwest::multipart::Part;

  use crate::drivers::bili::GetCsrfError;
  use crate::encoder::Encoder;

  use super::BiliClient;

  #[tokio::test]
  async fn get_csrf_test() {
    let bili_cli = BiliClient::new().await;
    match bili_cli.get_csrf().await {
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
    let bili_cli = BiliClient::new().await;
    dbg!(bili_cli.get_login_qr().await.unwrap());
  }

  #[tokio::test]
  async fn qr_code_login_test() {
    if option_env!("RUN_MANUALLY") != Some("true") {
      return;
    }
    let bili_cli = BiliClient::new().await;
    let rsp = bili_cli.get_login_qr().await.unwrap();
    qr2term::print_qr(rsp.data.as_ref().unwrap().url.as_ref().unwrap()).unwrap();
    let resp = bili_cli.login_qrcode(&rsp).await;
    bili_cli.save_cookies().await.unwrap();
    dbg!(resp.unwrap());
  }

  #[tokio::test]
  async fn get_self_info_test() {
    let bili_cli = BiliClient::new().await;
    let rsp = bili_cli.get_self_info().await.unwrap();
    dbg!(rsp);
  }

  #[tokio::test]
  async fn upload_image_via_album_test() {
    let bili_cli = BiliClient::new().await;
    let enc = crate::encoder::png::PngEncoder();
    let encoded = enc.encode(&[80; 114514]).unwrap();
    let rsp = bili_cli
      .upload_image_via_album(Part::bytes(encoded))
      .await
      .unwrap();
    dbg!(rsp);
  }
}

const QUOTES: [&str; 13] = [
  "叔叔我啊，真的要生气了",
  "你所热爱的，就是你的生活",
  "你妈什么时候死啊?",
  "我没改变B站",
  "LV4会员每年将免费获得3个月的“大会员”",
  "LV5会员每年将免费获得6个月的“大会员”",
  "LV6会员每年将免费获得9个月的“大会员",
  "B站未来有可能会倒闭，但绝不会变质",
  "资本把周宇翔变成了鬼",
  "汉奸是因利益而产生的，和过去或现在无关",
  "我没改变B站",
  "2021年准备让b站市值翻几倍?",
  "诶哟哟，我一般是不看市值，最重要的是用户满意度",
];
