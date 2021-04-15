use async_trait::async_trait;
use bytes::Bytes;
use crate::types::blob::{Blob};
use crate::types::errors::{
    BucketResult,
    BlobResult};


/// Collection of buckets, can be used to list, create
/// open, delete buckets
#[async_trait]
pub trait Buckets<T, P> 
    where T: Bucket<P>, P: Blob{
    /// Open an existing bucket 
    async fn open(&mut self, bucket_name: String) -> BucketResult<T>;
    /// Create a bucket at location
    async fn create(&mut self, bucket_name: String, location: Option<String>) -> BucketResult<T>;
    /// List all buckets
    async fn list(&mut self) -> Vec<T>;
    /// Delete a bucket
    async fn delete(&mut self, bucket_name: String) -> BucketResult<bool>;
    /// Check if a bucket exists
    async fn exists(&mut self, bucket_name: String) -> bool;
}

/// Bucket delete single object, can create blob,
/// delete blob and retrieve blob
#[async_trait]
pub trait Bucket<P>
    where P: Blob{
//    /// List all blobs
//    /// Returns Ok((Vec<P>, Option<String>)) where Option<String> is the
//    /// next marker to use in listing blobs
//    async fn list_blobs(&self, marker: Option<String>) -> BucketResult<(Vec<P>, Option<String>)>;
//    /// Retrieve a blob from this bucket
//    /// Specify blob_path e.g "pictures/image1.png"
//    /// content_range is range to retrieve at once, if None, retrieve entire object
//    async fn get_blob(&self, blob_path: String, content_range: Option<String>) -> BlobResult<P>;
//    /// copy blob_path to another blob path
//    /// blob_destination_path is formated as {bucket_name}/{path}
//    /// e.g bucket1/folder/simple.jpeg
//    /// specify content_type for destination file
//    async fn copy_blob(&self,
//                       blob_path: String, 
//                       blob_destination_path: String,
//                       content_type: Option<String>) -> BlobResult<P>;
//    /// Write to a blob in bucket
//    async fn write_blob(&self, blob_name: String, content: Option<Bytes>) -> BlobResult<P>;
//    /// Delete a blob from bucket
//    async fn delete_blob(&self, blob_path: String) -> BlobResult<bool>;
}
