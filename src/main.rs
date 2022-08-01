#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;

use std::env::set_current_dir;
use std::path::Path;
use std::{env, fs};

use chrono::Local;
use tracing::{debug, warn, Level};
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::{fmt, EnvFilter, FmtSubscriber};

mod dirs;
mod encoder;

fn main() {
  set_debug_work_dir();
  init_logger();
  if cfg!(debug_assertions) {
    warn!("This is a DEBUG build for Cutis, for development purposes only.")
  }
  debug!("Cutis is working on {:?}", env::current_dir());
}

fn set_debug_work_dir() {
  if !cfg!(debug_assertions) {
    return;
  }
  let cur_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
  let buf = cur_dir.join("./cutis-work");
  fs::create_dir_all(&buf).expect(&*format!("Failed to create folder {buf:?}"));
  set_current_dir(buf).expect("set_current_dir failed");
}

fn init_logger() {
  struct LocalTimer;

  impl FormatTime for LocalTimer {
    fn format_time(&self, w: &mut fmt::format::Writer<'_>) -> std::fmt::Result {
      write!(w, "{}", Local::now().format("%FT%T%.3f"))
    }
  }

  let subscriber = FmtSubscriber::builder()
    .with_timer(LocalTimer)
    .with_thread_names(true)
    .with_env_filter(EnvFilter::from_env("CUTIS_LOGGER"))
    .with_ansi(true)
    .with_max_level(if cfg!(debug_assertions) {
      Level::TRACE
    } else {
      Level::INFO
    })
    .finish();
  tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}
