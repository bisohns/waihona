//! Rust library for cloud storage across major cloud providers
//! It aims to provide simple to use functions to perform CRUD operations on
//! buckets and blobs.
//! Waihona simply means storage in Hawaiian
//! 
//!  ## Feature Flags
//!  
//!  The following feature flags exist for this crate
//!  - [x] `aws`: Enable aws provider and dependencies
//!  - [x] `gcp`: Enable gcp provider and dependencies
//!  - [ ] `azure`: Enable azure provider and dependencies
//!
//!  ## Traits
//!
//!  Three major traits control behaviour for each provider 
//!
//!  Buckets -> Bucket -> Blob
//!  
//!```no_run
//! // all methods of traits are async
//!  use bytes::Bytes;
//!
//!  trait Buckets<T, P>     
//!      where T: Bucket<P>, P: Blob{    
//!          fn open(&mut self, bucket_name: &str);    
//!          fn create(&mut self, bucket_name: &str, location: Option<String>);
//!          fn list(&mut self);
//!          fn delete(&mut self, bucket_name: &str);
//!          fn exists(&mut self, bucket_name: &str);
//!     }       
//!
//! trait Bucket<P>
//!     where P: Blob{
//!         fn list_blobs(&self, marker: Option<String>);
//!         fn get_blob(&self, blob_path: &str, content_range: Option<String>);
//!         fn copy_blob(&self, blob_path: &str, blob_destination_path: &str, content_type: Option<String>);
//!         fn write_blob(&self, blob_name: &str, content: Option<Bytes>);
//!         fn delete_blob(&self, blob_path: &str); 
//!     }
//!
//!  trait Blob {
//!      fn delete(&self);
//!      fn copy(&self, blob_destination_path: &str, content_type: Option<String> );
//!      fn write(&self, content: Option<Bytes>); 
//!      fn read(&mut self);
//!     }
//! 
//!```
//!  
//! ## Examples
//!
//! These quick examples will show you how to make use of the
//! library for basic actions
//!
//!
//! List buckets from project waihona on GCP
//! 
//!
//! ```no_run
//! // ensure to export service credential using GOOGLE_APPLICATION_CREDENTIALS
//!#[cfg(feature = "gcp")]
//! use waihona::providers::gcp::GcpBucket;
//!
//!#[tokio::test]
//!#[cfg(feature = "gcp")]
//!async fn test_list_buckets() -> Vec<GcpBucket> {
//!    // Import Buckets trait from crate
//!    use waihona::types::bucket::{Buckets};
//!    use waihona::providers::gcp;
//!    let mut gcp_buckets = providers::gcp::GcpBuckets::new(
//!        "waihona"
//!        );
//!    // Returns (Vec<GcpBucket, Option<String>)
//!    // where Option<String> is the cursor for the token for next page listing
//!    let resp = gcp_buckets.list().await;
//!    resp[0]
//!}
//!```
//!
//!
//! Check bucket waihona exists on AWS
//!
//! ```no_run
//!
//!#[tokio::test]
//!#[cfg(feature = "aws")]
//!async fn test_bucket_exists() -> bool {
//!    use waihona::types::bucket::{Buckets};
//!    use waihona::providers;
//!    let mut aws_buckets = providers::aws::AwsBuckets::new(
//!        "us-east-2"
//!        );
//!    let resp = aws_buckets.exists(
//!        "waihona"
//!        ).await;
//!        // OR you can do
//!    let resp = providers::aws::AwsBucket::exists(
//!        "us-east-2",
//!        "waihona"
//!        ).await;
//!    resp
//!}
//!```
//!
//! Write content to a blob "example.txt" in waihona bucket on Azure
//!
//! ```no_run
//!#[cfg(feature = "azure")]
//! use waihona::providers::azure::AzureBlob;
//!
//!
//!
//!#[tokio::test]
//!#[cfg(feature = "azure")]
//!async fn test_create_blob() -> AzureBlob {
//!    // !! UNIMPLEMENTED
//!    use waihona::types::bucket::{Buckets, Bucket};
//!    use waihona::types::blob::{Blob};
//!    use waihona::providers;
//!    use bytes::Bytes;
//!    let mut azure_buckets = providers::azure::AzureBuckets::new();
//!    let waihona = azure_buckets.open(
//!        "waihona",
//!        ).await.unwrap();
//!    let mut blob = waihona.write_blob(
//!        "example.txt",
//!         Some(Bytes::from("Hello world"))
//!        ).await
//!        .unwrap();
//!     blob
//!  }
//!```
//!
//!  Copy file content from "example.txt" blob on AWS to blob on GCP
//!  and delete AWS blob afterwards
//!  assuming waihona buckets exist on both platforms
//!
//!```no_run
//!#[cfg(feature = "gcp")]
//! use waihona::providers::gcp::GcpBlob;
//!
//!
//!#[tokio::test]
//!#[cfg(all(feature = "gcp", feature = "aws" ))]
//!async fn test_transfer_blob() -> GcpBlob {
//!    use waihona::types::bucket::{Buckets, Bucket};
//!    use waihona::types::blob::{Blob};
//!    use waihona::providers;
//!    use bytes::Bytes;
//!    let mut aws_blob = providers::aws::AwsBlob::get(
//!        "us-east-2", // Region
//!        "waihona", // Bucket name
//!        "example.txt", // Blob name
//!        None // Content range
//!        ).await
//!        .unwrap();
//!    let mut gcp_blob = providers::gcp::GcpBlob::get(
//!        "gcp-project-name", // Project name
//!        "waihona", // Bucket name
//!        "example.txt", // Blob name
//!        None // Content range
//!        ).await
//!        .unwrap();
//!    let content: Bytes = aws_blob.read().unwrap();
//!    gcp_blob.write(Some(content)).await.unwrap();
//!     aws_blob.delete().unwrap();
//!     gcp_blob
//!  }
//!```

pub mod types;
pub mod providers;
#[cfg(test)]
pub mod tests;

