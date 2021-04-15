use async_trait::async_trait;
use crate::types::bucket::{Buckets, Bucket};
use crate::types::blob::{Blob};
use crate::types::errors::{
    BucketResult, BucketError, BlobResult,BlobError
};
use google_cloud::storage::{
    Bucket as StorageBucket, Client
};

//macro_rules! assert_ok {
//    ($expr:expr) => {
//        match $expr {
//            Ok(value) => value,
//            Err(err) => {
//                panic!("asserted result is an error: {}", err);
//            }
//        }
//    };
//}

#[derive(Clone)]
pub struct GcpBuckets {
    pub client: Client,
}

/// project name on GCP
/// Will make use of the exported credential at 
/// GOOGLE_APPLICATION_CREDENTIALS
impl GcpBuckets {
    pub async fn new(project_name: impl Into<String>) -> GcpBuckets {
        GcpBuckets{
            client: Client::new(project_name).await.unwrap(),
        }
    }

}

#[derive(Debug, Clone)]
pub struct GcpBlob {
}

#[async_trait]
impl Blob for GcpBlob {
}

#[async_trait]
impl Buckets<GcpBucket, GcpBlob> for GcpBuckets {
    async fn list(&mut self) -> Vec<GcpBucket> {
        let resp = self.client.buckets().await;
        let mut buckets: Vec<GcpBucket> = Vec::new();
        for bucket in resp.unwrap().iter() {
            let bucket_found = GcpBucket::new(
                String::from(bucket.name()),
                Some(self.client.clone()),
                None,
                Some(bucket.clone())
                ).await;
            buckets.push(bucket_found);
        }
        buckets
    }

    async fn open(&mut self, bucket_name: String) -> BucketResult<GcpBucket>{
        if self.exists(bucket_name.clone()).await {
            let bucket = match self.client.bucket(bucket_name.as_str()).await {
                Ok(b) => b,
                Err(_) => unreachable!()
            };

            Ok(
                GcpBucket::new(
                    bucket_name.clone(),
                    Some(self.client.clone()),
                    None,
                    Some(bucket)
                    ).await
                )
        } else {
            Err(BucketError::NotFound)
        }
    }

    async fn create(&mut self, bucket_name: String, _location: Option<String>) -> BucketResult<GcpBucket>{
        let resp = self.client.create_bucket(bucket_name.as_str()).await;
        match resp {
            Ok(a) => {
                Ok(GcpBucket::new(
                        bucket_name.clone(),
                        Some(self.client.clone()),
                        None,
                        Some(a)).await)
                    },
            Err(e) => {
                Err(BucketError::CreationError(
                    String::from(format!("{}",e))
                    ))
            },
        }

    }

    async fn delete(&mut self, bucket_name: String) -> BucketResult<bool> {
        match self.client.bucket(bucket_name.as_str()).await {
            Ok(b) => {
                b.delete();
                Ok(true)
            },
            Err(e) => {
                Err(BucketError::DeletionError(
                        String::from(format!("{}", e))
                        ))
            }
        }

    }
    
    async fn exists(&mut self, bucket_name: String) -> bool {
        match self.client.bucket(bucket_name.as_str()).await {
            Ok(_) => true,
            Err(_) => false
        }
    }


}

#[derive(Clone)]
pub struct GcpBucket {
    pub name: String,
    pub client: Client,
    pub bucket: Option<StorageBucket>
}

#[async_trait]
impl Bucket<GcpBlob> for GcpBucket {
}

impl GcpBucket {
    pub async fn new(name: String, client: Option<Client>, project_name: Option<String>, bucket: Option<StorageBucket>) -> GcpBucket {
        match client {
            Some(a) => GcpBucket{name, client:a, bucket},
            None => {
                GcpBucket {
                    name,
                    client: Client::new(
                        project_name.as_ref().unwrap()
                        ).await.unwrap(),
                    bucket
                }
            }
        }
    }
}

