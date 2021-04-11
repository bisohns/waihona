use async_trait::async_trait;
use crate::types::blob::{Blob};
use crate::types::errors::{
    BucketError, 
    BlobError,
    BucketResult,
    BlobResult};


/// Collection of buckets, can be used to list, create
/// open, delete buckets
#[async_trait]
pub trait Buckets<T, P> 
    where T: Bucket<P>, P: Blob{
    /// Open an existing bucket 
    async fn open(&self, bucket_name: String) -> BucketResult<T>;
    /// Create a bucket at location
    async fn create(&self, bucket_name: String, location: Option<String>) -> BucketResult<T>;
    /// List all buckets
    async fn list(&self) -> Vec<T>;
    /// Delete a bucket
    async fn delete(&self, bucket_name: String) -> BucketResult<bool>;
    /// Check if a bucket exists
    async fn exists(&self, bucket_name: String) -> bool;
}

/// Bucket delete single object, can create blob,
/// delete blob and retrieve blob
#[async_trait]
pub trait Bucket<P>
    where P: Blob{
//    /// Delete this particular bucket
//    async fn delete(&self) -> BucketResult<bool>;
//    /// Retrieve a blob from this bucket
//    async fn get_blob<T>(&self, blob_name: String) -> BlobResult<T>
//        where T: Blob;
//    /// Create a blob in bucket
//    async fn create_blob<T>(&self, blob_name: String) -> BlobResult<T>
//        where T: Blob;
//    /// Delete a blob from bucket
//    async fn delete_blob(&self, blob_name: String) -> BlobResult<bool>;
//    /// Check if a blob exists in bucket
//    async fn exists(&self, blob_name: String) -> bool;
}
