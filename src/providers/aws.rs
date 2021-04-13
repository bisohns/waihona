use async_trait::async_trait;
use bytes::Bytes;
use crate::types::bucket::{Buckets, Bucket};
use futures::{StreamExt, TryStreamExt};
use crate::types::blob::{Blob};
use crate::types::errors::{
    BucketResult, BucketError, BlobResult,BlobError
};
use rusoto_core::{Region};
use rusoto_s3::{
    S3, S3Client, CreateBucketRequest, CreateBucketConfiguration,
    DeleteBucketRequest, ListObjectsRequest, GetObjectRequest, StreamingBody
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
    body: Option<StreamingBody>,
    content_type: Option<String>,
    content_range: Option<String>
}
impl AwsBlob {
    pub fn new(key: Option<String>, 
               e_tag: Option<String>, 
               size: Option<i64>, 
               body: Option<StreamingBody>, 
               content_type: Option<String>,
               content_range: Option<String>
               ) -> Self {
        AwsBlob {
            key,
            e_tag,
            size,
            body,
            content_type,
            content_range
        }

    }
}

#[async_trait]
impl Blob for AwsBlob {
    async fn read(&mut self) -> BlobResult<Bytes> {
        match self.body {
            Some(ref mut res) => {
                let body = res.map_ok(|b| bytes::BytesMut::from(&b[..]))
                    .try_concat()
                    .await
                    .unwrap();
                Ok(body.freeze())
            },
            None => {
                Err(BlobError::ReadError)
            }
        }
    }
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
    /// Each AwsBlob does not have a body/content_type/content_range
    /// as those can only be gotten via a get_blob request
    async fn list_blobs(&self, marker: Option<String>) -> BucketResult<(Vec<AwsBlob>, Option<String>)> {
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
                            obj.size,
                            None,
                            None,
                            None,
                            )
                        )
                }
                Ok((ret, k.next_marker))
            },
            Err(e) => {
                Err(BucketError::ListError(
                    String::from(format!("{}",e))
                    ))
            },
        }
    }

    async fn get_blob(&self, blob_path: String, content_range: Option<String>) -> BlobResult<AwsBlob> {
        let get_blob_req = GetObjectRequest{
            bucket: self.name.clone(),
            key: blob_path.clone(),
            range: content_range,
            ..Default::default()
        };
        let resp = self.s3.get_object(get_blob_req).await;
        match resp {
            Ok(k) => {
                let blob = AwsBlob::new(
                    Some(blob_path),
                    k.e_tag.clone(),
                    k.content_length.clone(),
                    k.body,
                    k.content_type,
                    k.content_range
                    );
                Ok(blob)
            },
            Err(e) => {
                Err(BlobError::GetError(
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
