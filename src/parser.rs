use std::ffi::{OsStr, OsString};
use std::ops::RangeBounds;
use std::str::FromStr;

use clap::builder::{RangedU64ValueParser, StringValueParser, TypedValueParser};
use clap::{Arg, Command, ErrorKind};
use regex::Regex;

#[derive(Copy, Clone, Debug)]
pub struct RangedBytesValueParser {
  pub ranged_u64_parser: RangedU64ValueParser,
}

lazy_static! {
  static ref RE: Regex = Regex::new(r#"(?i)^(?P<num>\d+)\s*(?P<unit>[km])?i?b?$"#).unwrap();
}

impl RangedBytesValueParser {
  pub fn new<B: RangeBounds<u64>>(range: B) -> RangedBytesValueParser {
    RangedBytesValueParser {
      ranged_u64_parser: RangedU64ValueParser::<u64>::new().range(range),
    }
  }
}

#[test]
fn test() {
  dbg!(RE.find("4 MiB"));
}

impl TypedValueParser for RangedBytesValueParser {
  type Value = u64;

  fn parse_ref(
    &self,
    cmd: &Command,
    arg: Option<&Arg>,
    value: &OsStr,
  ) -> Result<Self::Value, clap::Error> {
    let raw = StringValueParser::new().parse_ref(cmd, arg, value)?;
    let caps = if let Some(caps) = RE.captures(&*raw.trim()) {
      caps
    } else {
      return Err(clap::Error::raw(
        ErrorKind::InvalidValue,
        "Syntax: Number + Unit, Units: [K, M, Ki, Mi, KiB, MiB]".to_string(),
      ));
    };
    let multiple = if let Some(matched) = &caps.name("unit") {
      match matched.as_str().to_ascii_lowercase().as_str() {
        "m" => 1024 * 1024,
        "k" => 1024,
        _ => 1,
      }
    } else {
      1
    };
    let num = u64::from_str(&caps["num"]).map_err(|e| {
      clap::Error::raw(
        ErrorKind::InvalidValue,
        format!("Failed to parse number {}", e),
      )
    })?;
    let value = if let Some(result) = num.checked_mul(multiple) {
      result
    } else {
      return Err(clap::Error::raw(
        ErrorKind::InvalidValue,
        format!("Value too large, {} * {}", num, multiple),
      ));
    };
    self
      .ranged_u64_parser
      .parse(cmd, arg, OsString::from(value.to_string()))
  }
}
