use crate::types::bucket::{Bucket, Buckets};
use async_trait::async_trait;
use bytes::Bytes;
use regex::Regex;
//use futures::{StreamExt, TryStreamExt};
use crate::types::blob::Blob;
use crate::types::errors::{BlobError, BlobResult, BucketError, BucketResult};
use futures::TryStreamExt;
use rusoto_core::Region;
use rusoto_s3::{
    CopyObjectRequest, CreateBucketConfiguration, CreateBucketRequest,
    DeleteBucketRequest, DeleteObjectRequest, GetObjectRequest,
    ListObjectsRequest, PutObjectRequest, S3Client, StreamingBody, S3,
};

pub struct AwsBuckets {
    s3: S3Client,
}

pub struct AwsBucket {
    name: String,
    s3: S3Client,
}

pub fn string_to_region(reg: &str) -> BucketResult<Region> {
    match reg {
        "ap-east-1" => Ok(Region::ApEast1),
        "ap-northeast-1" => Ok(Region::ApNortheast1),
        "ap-northeast-2" => Ok(Region::ApNortheast2),
        "ap-northeast-3" => Ok(Region::ApNortheast3),
        "ap-south-1" => Ok(Region::ApSouth1),
        "ap-southeast-1" => Ok(Region::ApSoutheast1),
        "ap-southeast-2" => Ok(Region::ApSoutheast2),
        "ca-central-1" => Ok(Region::CaCentral1),
        "eu-central-1" => Ok(Region::EuCentral1),
        "eu-west-1" => Ok(Region::EuWest1),
        "eu-west-2" => Ok(Region::EuWest2),
        "eu-west-3" => Ok(Region::EuWest3),
        "eu-north-1" => Ok(Region::EuNorth1),
        "eu-south-1" => Ok(Region::EuSouth1),
        "me-south-1" => Ok(Region::MeSouth1),
        "sa-east-1" => Ok(Region::SaEast1),
        "us-east-1" => Ok(Region::UsEast1),
        "us-east-2" => Ok(Region::UsEast2),
        "us-west-1" => Ok(Region::UsWest1),
        "us-west-2" => Ok(Region::UsWest2),
        "us-goveast-1" => Ok(Region::UsGovEast1),
        "us-govwest-1" => Ok(Region::UsGovWest1),
        "cn-north-1" => Ok(Region::CnNorth1),
        "cn-northwest-1" => Ok(Region::CnNorthwest1),
        "af-south-1" => Ok(Region::AfSouth1),
        _ => Err(BucketError::NotFound),
    }
}

impl AwsBucket {
    pub fn new(name: String, s3: Option<S3Client>) -> Self {
        match s3 {
            Some(a) => AwsBucket { name, s3: a },
            None => AwsBucket {
                name,
                s3: S3Client::new(Region::UsEast2),
            },
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
    content_range: Option<String>,
    bucket: String,
}
impl AwsBlob {
    pub fn new(
        key: Option<String>,
        e_tag: Option<String>,
        size: Option<i64>,
        body: Option<StreamingBody>,
        content_type: Option<String>,
        content_range: Option<String>,
        bucket: String,
    ) -> Self {
        AwsBlob {
            key,
            e_tag,
            size,
            body,
            content_type,
            content_range,
            bucket,
        }
    }

    pub async fn get(
        region: &str,
        bucket: &str,
        blob_path: &str,
        content_range: Option<String>,
    ) -> BlobResult<Self> {
        let mut aws_buckets = AwsBuckets::new(region);
        let bucket_str = String::from(bucket);
        let bucket = aws_buckets.open(&bucket_str).await;
        match bucket {
            Ok(b) => b.get_blob(blob_path, content_range).await,
            Err(e) => Err(BlobError::GetError(e.to_string())),
        }
    }
}

#[async_trait]
impl Blob for AwsBlob {
    async fn delete(&self) -> BlobResult<bool> {
        let bucket = AwsBucket::new(self.bucket.clone(), None);
        let resp = bucket.delete_blob(self.key.as_ref().unwrap()).await;
        match resp {
            Ok(_) => Ok(true),
            Err(e) => {
                Err(BlobError::DeletionError(String::from(format!("{}", e))))
            }
        }
    }

    async fn copy(
        &self,
        blob_destination_path: &str,
        content_type: Option<String>,
    ) -> BlobResult<bool> {
        let bucket = AwsBucket::new(self.bucket.clone(), None);
        let resp = bucket
            .copy_blob(
                self.key.as_ref().unwrap(),
                blob_destination_path,
                content_type,
            )
            .await;
        match resp {
            Ok(_) => Ok(true),
            Err(e) => Err(BlobError::CopyError(String::from(format!("{}", e)))),
        }
    }

    async fn write(&self, content: Option<Bytes>) -> BlobResult<bool> {
        let bucket = AwsBucket::new(self.bucket.clone(), None);
        let resp = bucket.write_blob(self.key.as_ref().unwrap(), content).await;
        match resp {
            Ok(_) => Ok(true),
            Err(e) => {
                Err(BlobError::WriteError(String::from(format!("{}", e))))
            }
        }
    }

    async fn read(&mut self) -> BlobResult<Bytes> {
        match self.body {
            Some(ref mut res) => {
                let body = res
                    .map_ok(|b| bytes::BytesMut::from(&b[..]))
                    .try_concat()
                    .await
                    .unwrap();
                Ok(body.freeze())
            }
            None => Err(BlobError::ReadError),
        }
    }
}

impl AwsBuckets {
    pub fn new(region: &str) -> Self {
        let reg = string_to_region(region).unwrap();
        AwsBuckets {
            s3: S3Client::new(reg),
        }
    }
}

impl AwsBucket {
    pub async fn exists(location: &str, bucket: &str) -> bool {
        let mut buckets = AwsBuckets::new(location);
        buckets.exists(bucket).await
    }
}

#[async_trait]
impl Bucket<AwsBlob> for AwsBucket {
    /// Each AwsBlob does not have a body/content_type/content_range
    /// as those can only be gotten via a get_blob request
    async fn list_blobs(
        &self,
        marker: Option<String>,
    ) -> BucketResult<(Vec<AwsBlob>, Option<String>)> {
        let list_blob_req = ListObjectsRequest {
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
                    ret.push(AwsBlob::new(
                        obj.key.clone(),
                        obj.e_tag.clone(),
                        obj.size,
                        None,
                        None,
                        None,
                        self.name.clone(),
                    ))
                }
                Ok((ret, k.next_marker))
            }
            Err(e) => {
                Err(BucketError::ListError(String::from(format!("{}", e))))
            }
        }
    }

    async fn copy_blob(
        &self,
        blob_path: &str,
        blob_destination_path: &str,
        content_type: Option<String>,
    ) -> BlobResult<AwsBlob> {
        let copy_source = format!("{}/{}", self.name.clone(), blob_path);
        let re = Regex::new(r"(?P<bucket>.*?)/(?P<blob_path>.*)").unwrap();
        if let Some(captures) = re.captures(blob_destination_path) {
            let bucket = captures.name("bucket").unwrap().as_str().to_owned();
            let key = captures.name("blob_path").unwrap().as_str().to_owned();
            let copy_blob_req = CopyObjectRequest {
                bucket: bucket.clone(),
                key: key.clone(),
                copy_source,
                ..Default::default()
            };
            let resp = self.s3.copy_object(copy_blob_req).await;
            match resp {
                Ok(_) => Ok(AwsBlob::new(
                    Some(key),
                    None,
                    None,
                    None,
                    content_type,
                    None,
                    bucket,
                )),
                Err(e) => {
                    Err(BlobError::CopyError(String::from(format!("{}", e))))
                }
            }
        } else {
            return Err(BlobError::CopyError(String::from(
                r"Format blob_destination_path as {bucket}/{blob_path}",
            )));
        }
    }

    async fn write_blob(
        &self,
        blob_path: &str,
        content: Option<Bytes>,
    ) -> BlobResult<AwsBlob> {
        let put_blob_req = PutObjectRequest {
            bucket: self.name.to_owned(),
            key: blob_path.to_string(),
            body: Some(content.unwrap().to_vec().into()),
            ..Default::default()
        };
        let resp = self.s3.put_object(put_blob_req).await;
        match resp {
            Ok(k) => Ok(AwsBlob::new(
                Some(blob_path.to_string()),
                k.e_tag,
                None,
                None,
                None,
                None,
                self.name.to_owned(),
            )),
            Err(e) => {
                Err(BlobError::WriteError(String::from(format!("{}", e))))
            }
        }
    }

    async fn delete_blob(&self, blob_path: &str) -> BlobResult<bool> {
        let delete_blob_req = DeleteObjectRequest {
            bucket: self.name.clone(),
            key: blob_path.to_string(),
            ..Default::default()
        };
        let resp = self.s3.delete_object(delete_blob_req).await;
        match resp {
            Ok(_) => Ok(true),
            Err(e) => {
                Err(BlobError::DeletionError(String::from(format!("{}", e))))
            }
        }
    }

    async fn get_blob(
        &self,
        blob_path: &str,
        content_range: Option<String>,
    ) -> BlobResult<AwsBlob> {
        let get_blob_req = GetObjectRequest {
            bucket: self.name.clone(),
            key: blob_path.to_string(),
            range: content_range,
            ..Default::default()
        };
        let resp = self.s3.get_object(get_blob_req).await;
        match resp {
            Ok(k) => {
                let blob = AwsBlob::new(
                    Some(blob_path.to_string()),
                    k.e_tag.clone(),
                    k.content_length.clone(),
                    k.body,
                    k.content_type,
                    k.content_range,
                    self.name.clone(),
                );
                Ok(blob)
            }
            Err(e) => Err(BlobError::GetError(String::from(format!("{}", e)))),
        }
    }
}

#[async_trait]
impl Buckets<AwsBucket, AwsBlob> for AwsBuckets {
    async fn list(&mut self) -> Vec<AwsBucket> {
        let resp = self.s3.list_buckets().await.unwrap();
        let mut buckets: Vec<AwsBucket> = Vec::new();
        for bucket in resp.buckets.unwrap().iter() {
            if bucket.name.is_some() {
                let bucket_found = AwsBucket::new(
                    String::from(bucket.name.clone().unwrap()),
                    Some(self.s3.clone()),
                );
                buckets.push(bucket_found);
            }
        }
        buckets
    }

    async fn open(&mut self, bucket_name: &str) -> BucketResult<AwsBucket> {
        if self.exists(&bucket_name).await {
            Ok(AwsBucket {
                name: bucket_name.to_string(),
                s3: self.s3.clone(),
            })
        } else {
            Err(BucketError::NotFound)
        }
    }

    async fn create(
        &mut self,
        bucket_name: &str,
        location: Option<String>,
    ) -> BucketResult<AwsBucket> {
        let create_bucket_req = CreateBucketRequest {
            bucket: bucket_name.to_string(),
            create_bucket_configuration: Some(CreateBucketConfiguration {
                location_constraint: location,
            }),
            ..Default::default()
        };
        let resp = self.s3.create_bucket(create_bucket_req).await;
        match resp {
            Ok(_) => Ok(AwsBucket {
                name: bucket_name.to_string(),
                s3: self.s3.clone(),
            }),
            Err(e) => {
                Err(BucketError::CreationError(String::from(format!("{}", e))))
            }
        }
    }

    async fn delete(&mut self, bucket_name: &str) -> BucketResult<bool> {
        if self.exists(bucket_name).await {
            let delete_bucket_req = DeleteBucketRequest {
                bucket: bucket_name.to_string(),
                ..Default::default()
            };

            let resp = self.s3.delete_bucket(delete_bucket_req).await;
            match resp {
                Ok(_) => Ok(true),
                Err(e) => Err(BucketError::DeletionError(String::from(
                    format!("{}", e),
                ))),
            }
        } else {
            Err(BucketError::NotFound)
        }
    }

    async fn exists(&mut self, bucket_name: &str) -> bool {
        let bucket_list = self.list().await;
        for bucket in bucket_list {
            if bucket.name == bucket_name {
                return true;
            }
        }
        false
    }
}
