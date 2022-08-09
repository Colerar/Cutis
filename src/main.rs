#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;

use std::env;
#[cfg(debug_assertions)]
use std::env::set_current_dir;
use std::fmt::{Display, Formatter};
#[cfg(debug_assertions)]
use std::fs::create_dir_all;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use blake3::Hasher;
use bytes::{Buf, BufMut, BytesMut};
use chrono::{Duration, Local};
use clap::builder::{EnumValueParser, RangedU64ValueParser};
use clap::{
  crate_version, ArgGroup, Args, Command, CommandFactory, Parser, PossibleValue, Subcommand,
  ValueEnum,
};
use clap_verbosity_flag::{LogLevel, Verbosity};
use dialoguer::theme::ColorfulTheme;
use futures::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};

use reqwest::ClientBuilder;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;
use tokio::{join, spawn};
use tracing::{debug, error, info, warn};

use tracing::metadata::LevelFilter;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::{fmt, FmtSubscriber};

use crate::drivers::bili::BiliClient;
use crate::drivers::Driver;
use crate::encoder::png::PngEncoder;
use crate::encoder::Encoder;
use crate::parser::RangedBytesValueParser;

mod dirs;
mod drivers;
mod encoder;
mod parser;

#[cfg(debug_assertions)]
type DefaultLevel = DebugLevel;

#[cfg(not(debug_assertions))]
type DefaultLevel = clap_verbosity_flag::InfoLevel;

#[derive(Copy, Clone, Debug, Default)]
pub struct DebugLevel;

impl LogLevel for DebugLevel {
  fn default() -> Option<log::Level> {
    Some(log::Level::Debug)
  }
}

#[derive(Parser, Debug, Clone)]
#[clap(name = "Cutis", bin_name = "cutis", version, about, long_about = None)]
struct Cli {
  /// Print version information
  #[clap(short = 'V', long, value_parser)]
  version: bool,
  #[clap(flatten)]
  verbose: Verbosity<DefaultLevel>,
  #[clap(subcommand)]
  command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
  /// Upload your files to the driver you selected
  #[clap(alias = "u", arg_required_else_help(true))]
  Upload(Upload),
  /// Login to driver
  #[clap(alias = "l")]
  Login(Login),
}

#[derive(Args, Debug, Clone)]
struct Upload {
  /// Files to upload
  #[clap(short = 'I', value_parser, value_name = "DIR")]
  #[clap(value_hint = clap::ValueHint::DirPath, required = true, multiple_values = true)]
  includes: Vec<PathBuf>,
  /// Image driver
  #[clap(short, long, value_parser = EnumValueParser::<Drivers>::new(), default_value = "bili")]
  driver: Drivers,
  /// Block size
  #[clap(
    short = 'b',
    long = "block-size",
    value_parser,
    default_value = "4 MiB"
  )]
  // 12 KiB <= block_size <= 12 MiB
  #[clap(value_parser = RangedBytesValueParser::new(16 * 1024..=12 * 1024 * 1024))]
  block_size: u64,
  /// Max concurrent worker
  #[clap(short = 'c', long = "concurrent", default_value_t = 8)]
  #[clap(value_parser = RangedU64ValueParser::<u8>::new().range(2..=63))]
  max_conc: u8,
  /// Max retry times
  #[clap(short = 'r', long = "retry", value_parser, default_value_t = 3)]
  max_retry: u8,
}

#[derive(Args, Debug, Clone)]
#[clap(group(
  ArgGroup::new("ways")
    .required(true)
    .args(&["cookie", "qrcode"]),
))]
struct Login {
  /// Login via Cookie
  #[clap(short = 'c', long, value_parser)]
  cookie: bool,
  /// Login via scanning QrCode
  #[clap(short = 'Q', long = "qr", value_parser)]
  qrcode: bool,
  /// Which driver to login
  #[clap(short, long, value_parser = EnumValueParser::<Drivers>::new(), default_value = "bili")]
  driver: Drivers,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Drivers {
  Bili,
}

impl Drivers {
  async fn spawn_driver(&self) -> Box<dyn Driver + Sync + Send> {
    self.spawn_driver_with_options(|i| i).await
  }

  async fn spawn_driver_with_options<F>(&self, option: F) -> Box<dyn Driver + Sync + Send>
  where
    F: FnOnce(ClientBuilder) -> ClientBuilder + Send,
  {
    match &self {
      Drivers::Bili => {
        if let Ok(client) = BiliClient::new_with_options(option).await {
          Box::new(client)
        } else {
          exit(exitcode::SOFTWARE)
        }
      }
    }
  }
}

impl Display for Drivers {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.to_possible_value().unwrap().get_name())
  }
}

impl ValueEnum for Drivers {
  fn value_variants<'a>() -> &'a [Self] {
    &[Self::Bili]
  }

  fn to_possible_value<'a>(&self) -> Option<PossibleValue<'a>> {
    match self {
      Self::Bili => Some(PossibleValue::new("bilibili").aliases(vec!["bili"])),
    }
  }
}

#[tokio::main]
async fn main() {
  let args: Cli = Cli::parse();
  let mut cmd: Command = Cli::command();
  if args.version {
    println!("Cutis v{}", crate_version!());
    exit(0);
  }

  if args.command.as_ref().is_none() {
    cmd.print_help().expect("failed to write to stdout");
    exit(0);
  }

  #[cfg(debug_assertions)]
  set_debug_work_dir();
  init_logger(Some(&args));
  debug!("Args: {:?}", &args);
  #[cfg(debug_assertions)]
  warn!("This is a DEBUG build for Cutis, for development purposes only.");
  debug!("Cutis is working on {:?}", env::current_dir());

  clap::value_parser!(u32).range(16 * 1024..12 * 1024 * 1024);

  match args.command.unwrap() {
    Commands::Upload(subcmd) => {
      {
        let not_exists: Vec<_> = subcmd
          .includes
          .iter()
          .filter(|path| !path.exists())
          .map(|path| path.to_string_lossy())
          .collect();
        if !not_exists.is_empty() {
          error!("These files do not exist: {:#?}", not_exists);
          exit(exitcode::USAGE);
        }
        let not_files: Vec<_> = subcmd
          .includes
          .iter()
          .filter(|path| !path.is_file())
          .map(|path| path.to_string_lossy())
          .collect();
        if !not_files.is_empty() {
          error!("These paths are not file: {:#?}", not_files);
          if subcmd.includes.iter().any(|i| i.is_dir()) {
            error!("Currently, Cutis do not support upload directory directly.");
            error!("Please archive them as a file and upload it.");
          }
          exit(exitcode::USAGE);
        }
      }
      let driver = Arc::new(subcmd.driver.spawn_driver().await);
      if driver.upload_need_login() {
        match driver.is_login().await {
          Ok(is_login) => {
            if !is_login {
              error!("Not login to driver: {}", subcmd.driver);
              exit(exitcode::USAGE);
            }
          }
          Err(err) => {
            error!("{err:?}");
            exit(exitcode::SOFTWARE);
          }
        }
      }
      let mut stream = tokio_stream::iter(subcmd.includes.clone());
      while let Some(path) = stream.next().await {
        let result = upload(Arc::clone(&driver), path.clone(), &subcmd).await;
        if let Err(err) = result {
          error!("Failed to upload file: {}", path.to_string_lossy());
          error!("{err:?}");
        }
      }
    }
    Commands::Login(subcmd) => {
      let driver = subcmd.driver.spawn_driver().await;
      info!("Logging in to driver: {}", subcmd.driver);
      if subcmd.cookie {
        let dialog = dialoguer::Password::with_theme(&ColorfulTheme::default())
          .with_prompt("Your cookie here (input was hidden)")
          .interact();
        let cookie = if let Ok(cookie) = dialog {
          cookie
        } else {
          exit(exitcode::USAGE);
        };

        if let Err(err) = driver.cookie_login(&*cookie).await {
          error!("Failed to login with cookie: {err:?}");
        };

        driver.print_self_info().await;
      } else if subcmd.qrcode {
        if let Err(err) = driver.qr_login().await {
          error!("Failed to login with qrcode: {err:?}");
        };
      };
    }
  }
}

#[cfg(debug_assertions)]
fn set_debug_work_dir() {
  let cur_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
  let buf = cur_dir.join("./cutis-work");
  create_dir_all(&buf).expect(&*format!("Failed to create folder {buf:?}"));
  set_current_dir(buf).expect("set_current_dir failed");
}

fn init_logger(args: Option<&Cli>) {
  struct LocalTimer;

  impl FormatTime for LocalTimer {
    fn format_time(&self, w: &mut fmt::format::Writer<'_>) -> std::fmt::Result {
      write!(w, "{}", Local::now().format("%T"))
    }
  }

  fn convert_filter(filter: log::LevelFilter) -> LevelFilter {
    match filter {
      log::LevelFilter::Off => LevelFilter::OFF,
      log::LevelFilter::Error => LevelFilter::ERROR,
      log::LevelFilter::Warn => LevelFilter::WARN,
      log::LevelFilter::Info => LevelFilter::INFO,
      log::LevelFilter::Debug => LevelFilter::DEBUG,
      log::LevelFilter::Trace => LevelFilter::TRACE,
    }
  }

  let level = if let Some(args) = args {
    convert_filter(args.verbose.log_level_filter())
  } else if cfg!(debug_assertions) {
    LevelFilter::DEBUG
  } else {
    LevelFilter::INFO
  };

  let builder = FmtSubscriber::builder();
  let subscriber = builder
    .compact()
    .with_timer(LocalTimer)
    .with_max_level(level)
    .with_thread_names(false)
    .with_ansi(true)
    .finish();
  tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
struct FileIndex {
  name: String,
  size: u64,
  /// Checksum for **the whole raw file**
  b3checksum: String, // blake3 checksum
  blocks: Vec<Block>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
struct Block {
  index: u64,
  size: u64,
  url: String,
  /// Checksum for **raw block**, instead of encoded image
  b3checksum: String,
}

impl FileIndex {
  async fn encode_to_image<E>(&self, encoder: &E) -> Result<Vec<u8>>
  where
    E: Encoder,
  {
    // serde_json::to_vec -> [u32 - size][json bytes] -> image
    let mut buf = BytesMut::new();
    let json_bin = serde_json::to_vec(self).context("Failed to encode FileIndex to json")?;
    buf.put_u32(json_bin.len() as u32);
    buf.put_slice(&*json_bin);
    let json_image = encoder
      .encode(&*buf.to_vec())
      .await
      .context("Failed to encode FileIndex json to image")?;
    Ok(json_image)
  }

  async fn decode_from_image<D>(&self, decoder: D, data: Vec<u8>) -> Result<Self>
  where
    D: Encoder,
  {
    // image -> [u32 - size][json bytes] -> serde_json::from_vec
    let json_image = decoder
      .decode(&*data, data.len())
      .await
      .context("Failed to decode FileIndex data from image")?;
    let mut data = bytes::Bytes::from(json_image);
    if data.len() < 4 {
      return Err(anyhow!(
        "Failed to decode data, too small {} < 4",
        data.len()
      ));
    }
    let json_size = data.get_u32() as usize;
    let json_bin = data
      .get(0..json_size)
      .context("Failed to read json image as FileIndex json, out of bounds")?;
    serde_json::from_slice::<Self>(&*json_bin).context("Failed to deserialize FileIndex json")
  }
}

async fn upload(
  driver: Arc<Box<dyn Driver + Send + Sync + 'static>>,
  path: PathBuf,
  args: &Upload,
) -> Result<()> {
  let driver = Arc::new(driver);
  let block_size = args.block_size;
  let max_conc = args.max_conc - 1;
  let max_retry = args.max_retry;
  let retry_interval = Duration::seconds(10);
  let path = Arc::new(path);

  if !path.is_file() {
    return Err(anyhow!("File not exists: {}", &path.to_string_lossy()));
  }

  let file_name = path
    .file_name()
    .context("Cannot parse file_name...")?
    .to_string_lossy();
  let file = File::open(&*path)
    .with_context(|| format!("Failed to open file {}", &path.to_string_lossy()))?;
  let file_meta = file
    .metadata()
    .with_context(|| format!("Unable to get the metadata of file {path:?}"))?;
  let file_len = file_meta.len();

  let hasher = Arc::new(std::sync::RwLock::new(Hasher::new()));
  let mut buf_reader = BufReader::new(file);

  let (tx, mut rx) = mpsc::channel(max_conc as usize);

  let mp = MultiProgress::new();

  let tick_chars = "⠁⠂⠄⡀⢀⠠⠐⠈ ";
  let progress_chars = "#>-";
  let upload_sty = ProgressStyle::with_template(
    "{spinner} {elapsed_precise:.dim} [{bar:35.cyan/blue}] {pos:>7}/{len:7} {msg}",
  )
  .unwrap()
  .tick_chars(tick_chars)
  .progress_chars(progress_chars);
  let disk_sty = ProgressStyle::with_template(
    "{spinner} {elapsed_precise:.dim} [{bar:35.cyan/blue}] {bytes:^15} {msg}",
  )
  .unwrap()
  .tick_chars(tick_chars)
  .progress_chars(progress_chars);

  let duration = core::time::Duration::from_millis(500);
  let diskp = ProgressBar::new(file_meta.len());
  let block_total = (file_meta.len() as f64 / block_size as f64).ceil() as u64;
  let encodep = ProgressBar::new(block_total);
  let uploadp = ProgressBar::new(block_total);

  if tracing::enabled!(tracing::Level::DEBUG) {
    mp.set_draw_target(ProgressDrawTarget::hidden());
  };

  mp.add(diskp.clone());
  mp.add(encodep.clone());
  mp.add(uploadp.clone());

  diskp.set_style(disk_sty);
  diskp.enable_steady_tick(duration);
  uploadp.set_style(upload_sty.clone());
  uploadp.enable_steady_tick(duration);
  encodep.set_style(upload_sty);
  encodep.enable_steady_tick(duration);

  mp.is_hidden();

  let blocks = Arc::new(RwLock::new(Vec::new()));

  let mut index = 0;
  let encoded_num = Arc::new(AtomicUsize::new(0));

  {
    let sender = async {
      loop {
        diskp.set_message(format!("Reading block {index}..."));
        let mut block = vec![0; block_size as usize];
        diskp.inc(block.len() as u64);
        match buf_reader.read(&mut block) {
          Ok(0) => {
            diskp.clone().finish_with_message("Complete reading file");
            debug!("Reaches the end of file");
            drop(tx);
            debug!("Tx dropped");
            break;
          }
          Ok(n) => {
            debug!("Uploading block {index:0>4}");
            let driver = Arc::clone(&driver);
            let blocks = Arc::clone(&blocks);
            let hasher = Arc::clone(&hasher);
            let encodep = encodep.clone();
            let uploadp = uploadp.clone();
            let path = Arc::clone(&path);
            let encoded_num = Arc::clone(&encoded_num);

            let to_upload = {
              let block = &block[..n];
              let mut hasher = hasher.write().unwrap();
              hasher.update_rayon(block);
              let mut vec = Vec::with_capacity(n);
              vec.extend_from_slice(block);
              Box::new(vec)
            };
            let handle: JoinHandle<()> = spawn(async move {
              let block_checksum = {
                let block_checksum = blake3::hash(&to_upload).to_hex();
                debug!("Block {index:0>4} Checksum: {block_checksum}");
                block_checksum
              };
              let enc = PngEncoder();
              let encoded = enc
                .encode(&Box::clone(&to_upload))
                .await
                .with_context(|| {
                  format!(
                    "Failed to encode file {path} block {index:0>4}",
                    path = &path.to_string_lossy()
                  )
                })
                .unwrap();
              drop(to_upload);
              let encoded = bytes::Bytes::from(encoded);
              encoded_num.fetch_add(1, Ordering::AcqRel);
              encodep.inc(1);
              encodep.set_message(format!("Encoded block {index}..."));
              if encoded_num.load(Ordering::Acquire) as u64 >= block_total {
                encodep.finish_with_message("Complete encoding");
              }
              let try_upload = || async {
                let url = driver.upload_image(encoded.clone()).await;
                let url = url.with_context(|| format!("Failed to upload block {index}"))?;
                let ok: Result<Block> = Ok(Block {
                  index: index as u64,
                  size: n as u64,
                  b3checksum: block_checksum.to_string(),
                  url: url.to_string(),
                });
                ok
              };

              let mut retry_times = 1;
              while retry_times <= max_retry {
                match try_upload().await {
                  Ok(block) => {
                    debug!("Successfully uploaded block {index:0>4}: {}", block.url);
                    debug!("{block:#?}");
                    uploadp.inc(1);
                    uploadp.set_message(format!("Uploaded block {index}..."));
                    let mut blocks = blocks.write().await;
                    blocks.push(block);
                    break;
                  }
                  Err(err) => {
                    error!(
                      "Failed to upload, retry times: {retry_times} in {}",
                      retry_interval.to_string()
                    );
                    error!("{err:?}");
                    retry_times += 1;
                  }
                };
              }
            });
            tx.send(handle)
              .await
              .context("Failed send handle to Rx")
              .unwrap();
          }
          Err(err) => return Err(err).context("Failed to read, io error").unwrap(),
        }
        index += 1;
      }
    };

    let receiver = async {
      while let Some(job) = rx.recv().await {
        job.await.unwrap();
      }
    };

    join!(sender, receiver);

    uploadp.finish_with_message("Complete uploading");
  };

  let file_checksum = {
    let hasher = hasher.write().unwrap();
    hasher.finalize().to_hex()
  };

  let blocks = Arc::clone(&blocks);
  let guard = blocks.read().await;
  let mut blocks = guard.to_vec();
  blocks.sort_by(|a, b| a.index.cmp(&b.index));
  let file_index = FileIndex {
    name: file_name.to_string(),
    blocks,
    size: file_len,
    b3checksum: file_checksum.to_string(),
  };

  info!("All images are uploaded!");
  info!("Generating and uploading indexes...");

  debug!("{:?}", file_index);

  {
    let file_index_img = file_index
      .encode_to_image(&PngEncoder())
      .await
      .context("Failed to encode FileIndex to image")?;
    let file_index_img = bytes::Bytes::from(file_index_img);
    let mut retry = 1;
    while retry <= max_retry {
      match driver.upload_image(file_index_img.clone()).await {
        Ok(url) => {
          info!("Index url: {}", url);
          if let Some(short) = driver.abbr_url(url.as_str()) {
            info!("Short url: {short}");
          }
          break;
        }
        Err(err) => {
          error!("{}", err.context("Upload metadata failed..."));
          error!("Retry times: {}", retry)
        }
      };
      retry += 1;
    }
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use crate::{Block, FileIndex, PngEncoder};

  #[tokio::test]
  async fn file_index_enc_test() {
    let example = FileIndex {
      name: "test".to_string(),
      size: 1231232312123,
      b3checksum: "adfasdasdfasdfsadf".to_string(),
      blocks: vec![Block {
        index: 1,
        size: 0,
        url: "https://example.org".to_string(),
        b3checksum: "08abfcd110201".to_string(),
      }],
    };
    let encoded = example.encode_to_image(&PngEncoder()).await.unwrap();
    let decoded = example
      .decode_from_image(PngEncoder(), encoded)
      .await
      .unwrap();
    assert_eq!(decoded, example);
  }
}
