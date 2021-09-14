use async_trait::async_trait;
use bytes::Bytes;
use crate::types::errors::{BlobResult};

#[async_trait]
/// Blob can be used to write to blob, read from blob 
/// and delete blob
pub trait Blob {
    /// Delete blob
    async fn delete(&self) -> BlobResult<bool>;
    /// copy blob
    async fn copy(&self,
                  blob_destination_path: &str,
                  content_type: Option<String>
                  ) -> BlobResult<bool>;
    /// Write to blob
    async fn write(&self, content: Option<Bytes>) -> BlobResult<bool>;
    /// Read from blob
    async fn read(&mut self) -> BlobResult<Bytes>;
}
