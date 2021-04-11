use async_trait::async_trait;

#[async_trait]
/// Blob can be used to write to blob, read from blob 
/// and delete blob
pub trait Blob {
    /// Create a new blob object name
    async fn new(&self, blob_name: String) -> Self;
    /// Delete blob
    async fn delete(&self);
    /// Write to blob
    async fn write(&self);
    /// Read from blob
    async fn read(&self);
}
