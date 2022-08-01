use std::cmp::Ordering;
use std::io;
use std::io::{Cursor, Write};

use png::{BitDepth, ColorType, Compression, DecodingError, EncodingError};

use super::Encoder;

pub struct PngEncoder();

impl Encoder for PngEncoder {
  type Error = PngError;

  fn encode(&self, data: &[u8]) -> Result<Vec<u8>, Self::Error> {
    let modifier = MIB_MATRIX
      .iter()
      .find(|(size, _)| data.len() == *size)
      .or_else(|| MIB_MATRIX.iter().find(|(size, _)| data.len() <= *size))
      .map(|(_, func)| func);

    let mut metadata = Metadata::default();

    if let Some(func) = modifier {
      func(&mut metadata);
    } else {
      Err(PngError::TooLarge {
        len: data.len(),
        max: *MIB_MATRIX
          .iter()
          .rfind(|_| true)
          .map(|(size, _)| size)
          .unwrap(),
      })?;
    }

    let padding_err = || {
      Err(PngError::Padding {
        to_padding: metadata.byte_to_padding(data.len()),
        data_len: data.len(),
      })
    };

    metadata.compression = match metadata.padding_ratio(data.len()) {
      Some(ratio) => match (ratio * 100.0) as u32 {
        0..=30 => Compression::Fast,
        31..=70 => Compression::Default,
        71..=100 => Compression::Best,
        _ => padding_err()?,
      },
      _ => padding_err()?,
    };

    let mut cursor = Cursor::new(Vec::new());
    {
      let mut enc = png::Encoder::new(&mut cursor, metadata.width, metadata.height);
      enc.set_depth(metadata.bit_depth);
      enc.set_color(metadata.color_type);
      let mut writer = enc.write_header().map_err(PngError::Encoding)?;
      let mut stream = writer.stream_writer().map_err(PngError::Encoding)?;
      stream.write_all(data).map_err(PngError::Io)?;
      let to_pad_size = metadata.byte_to_padding(data.len()).unwrap();
      stream
        .write_all(&*vec![0; to_pad_size])
        .map_err(PngError::Io)?;
      stream.finish().map_err(PngError::Encoding)?;
    }
    Ok(cursor.into_inner())
  }

  fn decode(&self, data: &[u8], data_len: usize) -> Result<Vec<u8>, Self::Error> {
    let decoder = png::Decoder::new(data);
    let mut reader = decoder.read_info().map_err(PngError::Decoding)?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).map_err(PngError::Decoding)?;
    if info.buffer_size() < data_len {
      Err(PngError::OutOfBound {
        size: data_len,
        bound: info.buffer_size(),
      })?;
    }
    let pad_len = info.buffer_size() - data_len;
    Ok(buf[..info.buffer_size() - pad_len].to_vec())
  }
}

#[derive(thiserror::Error, Debug)]
pub enum PngError {
  #[error(
    "Input is too large to process, {} KiB / {} KiB ",
    *len as f64 / 1024.0,
    *max as f64 / 1024.0)
  ]
  TooLarge { len: usize, max: usize },
  #[error("Padding error occurred, to_padding: {to_padding:?}, data_len: {data_len}")]
  Padding {
    to_padding: Option<usize>,
    data_len: usize,
  },
  #[error("Size {size} out of bound {bound}")]
  OutOfBound { size: usize, bound: usize },
  #[error("An encoding error occurred during png enc/dec {0}")]
  Encoding(
    #[from]
    #[source]
    EncodingError,
  ),
  #[error("An encoding error occurred during png enc/dec {0}")]
  Decoding(
    #[from]
    #[source]
    DecodingError,
  ),
  #[error("An I/O error occurred during png enc/dec {0}")]
  Io(
    #[from]
    #[source]
    io::Error,
  ),
}

#[derive(Debug)]
struct Metadata {
  width: u32,
  height: u32,
  bit_depth: BitDepth,
  color_type: ColorType,
  compression: Compression,
}

impl Metadata {
  fn default() -> Metadata {
    Metadata {
      width: 1024,
      height: 1024,
      bit_depth: BitDepth::Sixteen,
      color_type: ColorType::Rgba,
      compression: Compression::Fast,
    }
  }

  fn size(&self) -> Option<usize> {
    let pixel_size_bit =
      bit_depth_size(&self.bit_depth) as usize * color_type_multiple(&self.color_type) as usize;
    let size = self.width as usize * self.height as usize * pixel_size_bit;
    if size % 8 != 0 {
      return None;
    }
    Some(size / 8)
  }

  fn byte_to_padding(&self, data_len: usize) -> Option<usize> {
    let target_size = self.size()?;
    match target_size.cmp(&data_len) {
      Ordering::Less => None,
      Ordering::Equal => Some(0),
      Ordering::Greater => Some(target_size - data_len),
    }
  }

  fn padding_ratio(&self, data_len: usize) -> Option<f64> {
    Some(self.byte_to_padding(data_len)? as f64 / self.size()? as f64)
  }
}

fn bit_depth_size(bit_depth: &BitDepth) -> u32 {
  match bit_depth {
    BitDepth::One => 1,
    BitDepth::Two => 2,
    BitDepth::Four => 4,
    BitDepth::Eight => 8,
    BitDepth::Sixteen => 16,
  }
}

fn color_type_multiple(color_type: &ColorType) -> u32 {
  match color_type {
    ColorType::Indexed => 1,
    ColorType::Grayscale => 1,
    ColorType::GrayscaleAlpha => 2,
    ColorType::Rgb => 3,
    ColorType::Rgba => 4,
  }
}

type MetadataBuilder = fn(&mut Metadata);

// (byte, func)
const MIB_MATRIX: [(usize, MetadataBuilder); 15] = [
  (32768 /* 16 KiB */, |i: &mut Metadata| {
    i.width = 64;
    i.height = 64;
    i.bit_depth = BitDepth::Eight; // 1 byte
    i.color_type = ColorType::Rgba; // 4
  }),
  (32768 /* 32 KiB */, |i: &mut Metadata| {
    i.width = 64;
    i.height = 128;
    i.bit_depth = BitDepth::Eight; // 1 byte
    i.color_type = ColorType::Rgba; // 4
  }),
  (65536 /* 64 KiB */, |i: &mut Metadata| {
    i.width = 128;
    i.height = 128;
    i.bit_depth = BitDepth::Eight; // 1 byte
    i.color_type = ColorType::Rgba; // 4
  }),
  (131072 /* 128 KiB */, |i: &mut Metadata| {
    i.width = 128;
    i.height = 256;
    i.bit_depth = BitDepth::Eight; // 1 byte
    i.color_type = ColorType::Rgba; // 4
  }),
  (262144 /* 256 KiB */, |i: &mut Metadata| {
    i.width = 256;
    i.height = 256;
    i.bit_depth = BitDepth::Eight; // 1 byte
    i.color_type = ColorType::Rgba; // 4
  }),
  (524_288 /* 512 KiB */, |i: &mut Metadata| {
    i.width = 256;
    i.height = 512;
    i.bit_depth = BitDepth::Eight; // 1 byte
    i.color_type = ColorType::Rgba; // 4
  }),
  (1024 * 1024, |i: &mut Metadata| {
    i.width = 512;
    i.height = 512;
    i.bit_depth = BitDepth::Eight; // 1 byte
    i.color_type = ColorType::Rgba; // 4
  }),
  (1_572_864 /* 1024 * 1024 * 1.5 */, |i: &mut Metadata| {
    i.width = 512;
    i.height = 1024;
    i.bit_depth = BitDepth::Eight; // 1 byte
    i.color_type = ColorType::Rgb; // 3
  }),
  (1024 * 1024 * 2, |i: &mut Metadata| {
    i.width = 1024;
    i.height = 1024;
    i.bit_depth = BitDepth::Eight; // 1 byte
    i.color_type = ColorType::GrayscaleAlpha; // 2
  }),
  (2_621_440 /* 1024*1024*2.5 */, |i: &mut Metadata| {
    i.width = 512;
    i.height = 1280;
    i.bit_depth = BitDepth::Eight; // 1 byte
    i.color_type = ColorType::Rgba; // 4
  }),
  (1024 * 1024 * 3, |i: &mut Metadata| {
    i.width = 1024;
    i.height = 1024;
    i.bit_depth = BitDepth::Eight; // 1 byte
    i.color_type = ColorType::Rgb; // 3
  }),
  (1024 * 1024 * 4, |i: &mut Metadata| {
    i.width = 1024;
    i.height = 1024;
    i.bit_depth = BitDepth::Eight; // 1 byte
    i.color_type = ColorType::Rgba; // 4
  }),
  (1024 * 1024 * 6, |i: &mut Metadata| {
    i.width = 1024;
    i.height = 1024;
    i.bit_depth = BitDepth::Sixteen; // 2 byte
    i.color_type = ColorType::Rgb; // 3
  }),
  (1024 * 1024 * 8, |i: &mut Metadata| {
    i.width = 1024;
    i.height = 1024;
    i.bit_depth = BitDepth::Sixteen; // 2 byte
    i.color_type = ColorType::Rgba; // 4
  }),
  (1024 * 1024 * 12, |i: &mut Metadata| {
    i.width = 1024;
    i.height = 1536;
    i.bit_depth = BitDepth::Sixteen; // 2 byte
    i.color_type = ColorType::Rgba; // 4
  }),
];

#[cfg(test)]
mod tests {
  use std::sync::Mutex;
  use std::thread::{spawn, JoinHandle};

  use png::{BitDepth, ColorType, Compression};

  use crate::encoder::png::{Metadata, PngEncoder};
  use crate::encoder::Encoder;

  #[test]
  fn encode_png() {
    let enc = PngEncoder();
    enc.encode(&[1, 2, 3, 4]).unwrap();
  }

  #[test]
  fn to_padding_byte_should_work() {
    let metadata = Metadata {
      width: 10,
      height: 10,
      bit_depth: BitDepth::Sixteen,
      color_type: ColorType::Rgba,
      compression: Compression::Fast,
    };
    assert_eq!(metadata.byte_to_padding(700), Some(100));
    assert_eq!(metadata.byte_to_padding(800), Some(0));
    assert_eq!(metadata.byte_to_padding(900), None);

    let metadata = Metadata {
      width: 10,
      height: 3,
      bit_depth: BitDepth::Two,
      color_type: ColorType::Rgb,
      compression: Compression::Fast,
    };
    assert_eq!(metadata.byte_to_padding(2), None);
  }

  #[test]
  fn encode_different_size() {
    const ARR: [usize; 9] = [
      524_288,
      1024 * 1024,
      1_572_864,
      1024 * 1024 * 2,
      2_621_440,
      1024 * 1024 * 3,
      1024 * 1024 * 4,
      1024 * 1024 * 6,
      1024 * 1024 * 8,
    ];

    let list: Mutex<Vec<JoinHandle<()>>> = Mutex::new(vec![]);

    ARR.iter().for_each(|i| {
      let thread = spawn(|| {
        let enc = PngEncoder();
        enc.encode(&vec![0; *i]).unwrap();
      });
      let mut list = list.lock().unwrap();
      list.push(thread);
    });

    ARR.iter().for_each(|i| {
      let thread = spawn(|| {
        let enc = PngEncoder();
        enc.encode(&vec![0; *i - 1000]).unwrap();
      });
      let mut list = list.lock().unwrap();
      list.push(thread);
    });

    let mut list = list.into_inner().unwrap();

    while let Some(cur_thread) = list.pop() {
      cur_thread.join().unwrap();
    }
  }
}
