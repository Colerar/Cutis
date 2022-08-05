use async_trait::async_trait;

pub mod png;

#[async_trait]
pub trait Encoder {
  /// Encode data to png binary with padding
  async fn encode(&self, data: &[u8]) -> anyhow::Result<Vec<u8>>;

  /// Decode real data from png binary `data` with certain length
  ///
  /// # Arguments
  ///
  /// * `data_len` - Real data length, without padding
  async fn decode(&self, data: &[u8], length: usize) -> anyhow::Result<Vec<u8>>;
}
