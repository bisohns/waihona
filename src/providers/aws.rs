use async_trait::async_trait;
use crate::types::bucket::{Buckets, Bucket};
use crate::types::blob::{Blob};
use crate::types::errors::{BucketResult, BucketError};
use rusoto_core::{Region};
use rusoto_s3::{
    S3, S3Client, CreateBucketRequest, CreateBucketConfiguration,
    DeleteBucketRequest, ListObjectsRequest,

};

pub struct AwsBuckets{
    s3: S3Client,
}

pub struct AwsBucket{
    name: String,
    s3: S3Client,
}

impl AwsBucket {
    pub fn new(name: String, s3: S3Client) -> Self {
        AwsBucket {
            name,
            s3,
        }

    }
}

#[derive(Debug)]
pub struct AwsBlob {
    key: Option<String>,
    e_tag: Option<String>,
    size: Option<i64>,
}
impl AwsBlob {
    pub fn new(key: Option<String>, e_tag: Option<String>, size: Option<i64>) -> Self {
        AwsBlob {
            key,
            e_tag,
            size,
        }

    }
}

#[async_trait]
impl Blob for AwsBlob {
}


impl AwsBuckets {
    pub fn new(region: Region) -> Self {
        AwsBuckets {
            s3: S3Client::new(region)
        }

    }

}

#[async_trait]
impl Bucket<AwsBlob> for AwsBucket {
    async fn list_blobs(&self, marker: Option<String>) -> BucketResult<Vec<AwsBlob>> {
        let list_blob_req = ListObjectsRequest{
            bucket: self.name.clone(),
            marker,
            ..Default::default()
        };
        let resp = self.s3.list_objects(list_blob_req).await;
        match resp {
            Ok(k) => {
                let contents = k.contents.unwrap();
                let mut ret: Vec<AwsBlob> = Vec::new();
                for obj in contents.iter() {
                    ret.push(
                        AwsBlob::new(
                            obj.key.clone(),
                            obj.e_tag.clone(),
                            obj.size
                            )
                        )
                }
                Ok(ret)
            },
            Err(e) => {
                Err(BucketError::ListError(
                    String::from(format!("{}",e))
                    ))
            },
        }
    }
}

#[async_trait]
impl Buckets<AwsBucket, AwsBlob> for AwsBuckets {
    async fn list(&self) -> Vec<AwsBucket> {
        let resp = self.s3.list_buckets().await.unwrap();
        let mut buckets: Vec<AwsBucket> = Vec::new();
        for bucket in resp.buckets.unwrap().iter() {
            if bucket.name.is_some(){
                let bucket_found = AwsBucket::new(
                    String::from(bucket.name.clone().unwrap()),
                    self.s3.clone()
                    );
                buckets.push(bucket_found);
            }
        }
        buckets
    }

    async fn open(&self, bucket_name: String) -> BucketResult<AwsBucket>{
        if self.exists(bucket_name.clone()).await {
            Ok(AwsBucket{
                name: bucket_name.clone(),
                s3: self.s3.clone(),
            })
        } else {
            Err(BucketError::NotFound)
        }
    }

    async fn create(&self, bucket_name: String, location: Option<String>) -> BucketResult<AwsBucket>{
        let create_bucket_req = CreateBucketRequest{
            bucket: bucket_name.clone(),
            create_bucket_configuration: Some(CreateBucketConfiguration{
                location_constraint: location
            }),
            ..Default::default()
        };
        let resp = self.s3.create_bucket(create_bucket_req).await;
        match resp {
            Ok(_) => Ok(AwsBucket{
                name: bucket_name.clone(),
                s3: self.s3.clone()
            }),
            Err(e) => {
                Err(BucketError::CreationError(
                    String::from(format!("{}",e))
                    ))
            },
        }

    }

    async fn delete(&self, bucket_name: String) -> BucketResult<bool> {
        if self.exists(bucket_name.clone()).await {
            let delete_bucket_req = DeleteBucketRequest{
                bucket: bucket_name.clone(),
                ..Default::default()
            };

            let resp = self.s3.delete_bucket(delete_bucket_req).await;
            match resp {
                Ok(_) => Ok(true),
                Err(e) => {
                    Err(BucketError::DeletionError(
                            String::from(format!("{}", e))
                            ))
                },
            }
        } else {
            Err(BucketError::NotFound)
        }

    }
    
    async fn exists(&self, bucket_name: String) -> bool {
        let bucket_list = self.list().await;
        for bucket in bucket_list {
            if bucket.name == bucket_name {
                return true
            }
        }
        false
    }

}
