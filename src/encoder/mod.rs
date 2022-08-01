pub mod png;

pub trait Encoder {
  type Error;

  /// Encode data to png binary with padding
  fn encode(&self, data: &[u8]) -> Result<Vec<u8>, Self::Error>;

  /// Decode real data from png binary `data` with certain length
  ///
  /// # Arguments
  ///
  /// * `data_len` - Real data length, without padding
  fn decode(&self, data: &[u8], length: usize) -> Result<Vec<u8>, Self::Error>;
}
