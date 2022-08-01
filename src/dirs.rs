use std::env::current_dir;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

lazy_static! {
  pub(super) static ref DEBUG_WORK_DIR: Option<PathBuf> = debug_work_dir();
  pub(super) static ref PROJECT: Option<ProjectDirs> = project_dir();
  pub(super) static ref CONFIG: PathBuf = config_dir();
  pub(super) static ref DATA: PathBuf = data_dir();
  pub(super) static ref CACHE: PathBuf = cache_dir();
}

pub(self) fn debug_work_dir() -> Option<PathBuf> {
  if !cfg!(debug_assertions) {
    return None;
  }
  let cur_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
  Some(cur_dir.join("./cutis-work"))
}

pub(self) fn project_dir() -> Option<ProjectDirs> {
  ProjectDirs::from("me", "colerar", "cutis")
}

pub(self) fn get_some_dir(name: &str, config_project_dir: fn(&ProjectDirs) -> &Path) -> PathBuf {
  let path = format!("./{name}");
  debug_work_dir()
    .map(|mut i| {
      i.push(&path);
      i
    })
    .or_else(|| {
      let dirs = project_dir()?;
      Some(config_project_dir(&dirs).to_path_buf())
    })
    .or_else(|| {
      let mut cur_dir = if let Ok(path) = current_dir() {
        path
      } else {
        return None;
      };
      cur_dir.push(&path);
      Some(cur_dir)
    })
    .expect(&*format!("Failed to get {name} dir"))
}

pub(self) fn config_dir() -> PathBuf {
  get_some_dir("config", |i| i.config_dir())
}

pub(self) fn data_dir() -> PathBuf {
  get_some_dir("data", |i| i.data_dir())
}

pub(self) fn cache_dir() -> PathBuf {
  get_some_dir("cache", |i| i.cache_dir())
}
