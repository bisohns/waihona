use async_trait::async_trait;
use crate::types::bucket::{Buckets, Bucket};
use crate::types::blob::{Blob};
use crate::types::errors::{
    BucketResult, BucketError, BlobResult,BlobError
};
use google_cloud::storage::{Client};

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
                None
                ).await;
            buckets.push(bucket_found);
        }
        buckets
    }

//    async fn open(&self, bucket_name: String) -> BucketResult<GcpBucket>{
//        if self.exists(bucket_name.clone()).await {
//            Ok(GcpBucket{
//                name: bucket_name.clone(),
//                s3: self.s3.clone(),
//            })
//        } else {
//            Err(BucketError::NotFound)
//        }
//    }

//    async fn create(&self, bucket_name: String, location: Option<String>) -> BucketResult<GcpBucket>{
//        let create_bucket_req = CreateBucketRequest{
//            bucket: bucket_name.clone(),
//            create_bucket_configuration: Some(CreateBucketConfiguration{
//                location_constraint: location
//            }),
//            ..Default::default()
//        };
//        let resp = self.s3.create_bucket(create_bucket_req).await;
//        match resp {
//            Ok(_) => Ok(GcpBucket{
//                name: bucket_name.clone(),
//                s3: self.s3.clone()
//            }),
//            Err(e) => {
//                Err(BucketError::CreationError(
//                    String::from(format!("{}",e))
//                    ))
//            },
//        }

//    }

//    async fn delete(&self, bucket_name: String) -> BucketResult<bool> {
//        if self.exists(bucket_name.clone()).await {
//            let delete_bucket_req = DeleteBucketRequest{
//                bucket: bucket_name.clone(),
//                ..Default::default()
//            };

//            let resp = self.s3.delete_bucket(delete_bucket_req).await;
//            match resp {
//                Ok(_) => Ok(true),
//                Err(e) => {
//                    Err(BucketError::DeletionError(
//                            String::from(format!("{}", e))
//                            ))
//                },
//            }
//        } else {
//            Err(BucketError::NotFound)
//        }

//    }
    
//    async fn exists(&self, bucket_name: String) -> bool {
//        let bucket_list = self.list().await;
//        for bucket in bucket_list {
//            if bucket.name == bucket_name {
//                return true
//            }
//        }
//        false
//    }

}

#[derive(Clone)]
pub struct GcpBucket {
    pub name: String,
    pub client: Client
}

#[async_trait]
impl Bucket<GcpBlob> for GcpBucket {
}

impl GcpBucket {
    pub async fn new(name: String, client: Option<Client>, project_name: Option<String>) -> GcpBucket {
        match client {
            Some(a) => GcpBucket{name, client:a},
            None => {
                GcpBucket {
                    name,
                    client: Client::new(
                        project_name.as_ref().unwrap()
                        ).await.unwrap()
                }
            }
        }
    }
}

