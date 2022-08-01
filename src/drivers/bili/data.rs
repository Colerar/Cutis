use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct QrRsp {
  pub code: i32,
  pub status: Option<bool>,
  #[serde(rename = "ts")]
  pub timestamp: Option<u64>,
  pub data: Option<QrData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QrData {
  pub url: Option<String>,
  pub oauth_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginQrRsp {
  pub code: i32,
  pub message: Option<String>,
  #[serde(rename = "ts")]
  pub timestamp: Option<u64>,
  pub status: Option<bool>,
  data: RawLoginQrRspData,
}

impl LoginQrRsp {
  pub fn data(&self) -> Option<LoginQrData> {
    let data = self.data.0.clone()?;
    let i64 = data.as_i64();
    let url = data
      .as_object()
      .and_then(|map| map.get("url"))
      .and_then(|value| value.as_str())
      .map(|str| str.to_string());
    Some(LoginQrData {
      success: url.is_some(),
      failed_code: i64,
      url,
    })
  }
}

#[derive(Debug)]
pub struct LoginQrData {
  pub success: bool,
  pub failed_code: Option<i64>,
  pub url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RawLoginQrRspData(Option<serde_json::Value>);

#[derive(Debug, Deserialize)]
pub struct SelfInfoRsp {
  pub code: i32,
  pub message: Option<String>,
  pub ttl: Option<i32>,
  pub data: SelfInfo,
}

#[derive(Debug, Deserialize)]
pub struct SelfInfo {
  #[serde(rename = "isLogin")]
  pub is_login: bool,
  #[serde(rename = "face")]
  pub avatar: Option<String>,
  pub mid: Option<u64>,
  #[serde(rename = "uname")]
  pub username: Option<String>,
  // and so on... ignore them
}

#[derive(Debug, Deserialize)]
pub struct AlbumUploadRsp {
  pub code: i32,
  pub message: Option<String>,
  pub data: Option<AlbumUploadData>,
}

#[derive(Debug, Deserialize)]
pub struct AlbumUploadData {
  pub image_url: Option<String>,
  pub image_width: Option<u32>,
  pub image_height: Option<u32>,
}

#[cfg(test)]
mod tests {
  use super::LoginQrRsp;

  fn de_qr_login_rsp(json: &str) {
    let rsp: LoginQrRsp = serde_json::from_str(json).unwrap();
    dbg!(&rsp);
    dbg!(&rsp.data());
  }

  #[test]
  fn de_qr_login_rsp_failed_data() {
    de_qr_login_rsp(r#"{"code":1,"message":"asdfa","ts":123123,"status":true,"data":-100}"#);
  }

  #[test]
  fn de_qr_login_rsp_success_data() {
    de_qr_login_rsp(
      r#"{"code":1,"message":"asdfa","ts":123123,"status":true,"data":{"url":"asdfasdf"}}"#,
    );
  }

  #[test]
  fn de_qr_login_rsp_null_data() {
    de_qr_login_rsp(r#"{"code":1,"message":"asdfa","ts":123123,"status":true,"data": null}"#);
  }
}
