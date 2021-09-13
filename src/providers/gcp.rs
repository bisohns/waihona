use async_trait::async_trait;
use regex::Regex;
use futures::{StreamExt};
use bytes::Bytes;
use crate::types::bucket::{Buckets, Bucket};
use crate::types::blob::{Blob};
use crate::types::errors::{
    BucketResult, BucketError, BlobResult,BlobError
};

use cloud_storage::Client;
use cloud_storage::bucket::{NewBucket};
use cloud_storage::ListRequest;
use cloud_storage::object::ObjectList;
use cloud_storage::Result as CResult;


#[derive(Debug)]
pub struct GcpBuckets {
    pub client: Client,
    pub user_project: String,
}

/// project name on GCP
/// Will make use of the exported credential at 
/// GOOGLE_APPLICATION_CREDENTIALS
impl GcpBuckets {
    pub fn new(project_name: impl Into<String>) -> GcpBuckets {
        GcpBuckets{
            client: Client::default(),
            user_project: project_name.into(),
        }
    }

}

#[derive(Debug)]
pub struct GcpBlob {
    key: Option<String>,
    e_tag: Option<String>,
    size: Option<i64>,
    body: Option<Vec<u8>>,
    content_type: Option<String>,
    content_range: Option<String>,
    bucket: String,
}

#[async_trait]
impl Blob for GcpBlob {
    async fn delete(&self) -> BlobResult<bool> {
        unimplemented!();
    }
    
    async fn copy(&self,
                  blob_destination_path: String,
                  content_type: Option<String>
                  ) -> BlobResult<bool> {

        unimplemented!();
    }

    async fn write(&self, content: Option<Bytes>) -> BlobResult<bool> {
        unimplemented!();
    }

    async fn read(&mut self) -> BlobResult<Bytes> {
        unimplemented!();
    }

}


#[async_trait]
impl Buckets<GcpBucket, GcpBlob> for GcpBuckets {
    async fn list(&mut self) -> Vec<GcpBucket> {
        let resp = self.client.bucket().list().await;
        let mut buckets: Vec<GcpBucket> = Vec::new();
        for bucket in resp.unwrap().iter() {
            let bucket_found = GcpBucket{
                name: bucket.name.clone(),
                client: Client::default(),
                user_project: self.user_project.clone(),
                e_tag: bucket.etag.clone(),
                self_link: bucket.self_link.clone(),
            };
            buckets.push(bucket_found);
        }
        buckets
    }

    async fn open(&mut self, bucket_name: String) -> BucketResult<GcpBucket>{
        if self.exists(bucket_name.clone()).await {
            match self.client.bucket().read(bucket_name.as_str()).await {
                Ok(b) => {
                    Ok(GcpBucket{
                        name: b.name.clone(),
                        client: Client::default(),
                        user_project: self.user_project.clone(),
                        e_tag: b.etag.clone(),
                        self_link: b.self_link.clone(),
                    })
                },
                Err(_) => Err(BucketError::OpenError(
                        String::from("Could not open bucket")
                        ))
            }
        } else {
            Err(BucketError::NotFound)
        }
    }

    async fn create(&mut self, bucket_name: String, _location: Option<String>) -> BucketResult<GcpBucket>{
        let new_bucket = NewBucket {
            name: bucket_name.clone(),
            ..Default::default()
        };
        let resp = self.client.bucket().create(&new_bucket).await;
        match resp {
            Ok(a) => {
                Ok(GcpBucket{
                    name: a.name.clone(),
                    client: Client::default(),
                    user_project: self.user_project.clone(),
                    e_tag: a.etag.clone(),
                    self_link: a.self_link.clone(),
                })
                    },
            Err(e) => {
                Err(BucketError::CreationError(
                    String::from(format!("{}",e))
                    ))
            },
        }

    }

    async fn delete(&mut self, bucket_name: String) -> BucketResult<bool> {
        if self.exists(bucket_name.clone()).await {
            let bucket = self.client.bucket().read(bucket_name.as_str()).await.unwrap();
            match self.client.bucket().delete(bucket).await {
                Ok(_) => {
                    Ok(true)
                },
                Err(e) => {
                    Err(BucketError::DeletionError(
                            String::from(format!("{}", e))
                            ))
                }
            }
        } else {
            Ok(false)
        }
    }
    
    async fn exists(&mut self, bucket_name: String) -> bool {
        match self.client.bucket().read(bucket_name.as_str()).await {
            Ok(_) => true,
            Err(_) => false
        }
    }


}

#[derive(Debug)]
pub struct GcpBucket {
    pub name: String,
    pub client: Client,
    pub user_project: String,
    pub e_tag: String,
    pub self_link: String,
}

#[async_trait]
impl Bucket<GcpBlob> for GcpBucket {

    async fn list_blobs(&self, marker: Option<String>) -> BucketResult<(Vec<GcpBlob>, Option<String>)>{
        let all_objects = self.client.object().list(
            self.name.as_str(), 
            ListRequest {
                page_token: marker,
                ..Default::default()
            }
            ).await;
        match all_objects {
            Ok(object_list) => {
                let mut ret: Vec<GcpBlob> = Vec::new();
                let obj_stream = object_list.take(1).collect::<Vec<CResult<ObjectList>>>().await;
                let bckt = obj_stream[0].as_ref().unwrap();
                for obj in &bckt.items {
                    ret.push(
                        GcpBlob {
                            key: Some(obj.name.clone()),
                            e_tag: Some(obj.etag.clone()),
                            size: Some(obj.size as i64),
                            body: None,
                            content_type: obj.content_type.clone(),
                            content_range: None,
                            bucket: self.name.clone()
                        })
                }
                Ok((ret, None))
            },
            Err(_) => Err(BucketError::ListError(String::from("could not list")))
        }
    } async fn get_blob(&self, blob_path: String, content_range: Option<String>) -> BlobResult<GcpBlob>{
        let resp = self.client.object().read(
            self.name.as_str(), 
            blob_path.as_str(),
            ).await;
        match resp {
            Ok(k) => {
                let blob = GcpBlob{
                    key: Some(k.name.clone()),
                    e_tag: Some(k.etag.clone()),
                    size: Some(k.size as i64),
                    body: None,
                    content_type: k.content_type,
                    content_range: content_range,
                    bucket: self.name.clone()
                };
                Ok(blob)
            },
            Err(e) => {
                Err(BlobError::GetError(
                    String::from(format!("{}",e))
                    ))
            },
        }
    }

    async fn copy_blob(&self,
                       blob_path: String, 
                       blob_destination_path: String,
                       content_type: Option<String>) -> BlobResult<GcpBlob>{
        let copy_source = format!("{}/{}", self.name.clone(), blob_path.clone());
        let re = Regex::new(r"(?P<bucket>.*?)/(?P<blob_path>.*)").unwrap();
        if let Some(captures) = re.captures(&blob_destination_path[..]) {
            let bucket = captures
                .name("bucket")
                .unwrap().as_str().to_owned();
            let key = captures
                .name("blob_path")
                .unwrap().as_str().to_owned();
            let obj = self.client.object().read(
                self.name.as_str(), 
                blob_path.as_str(),
            ).await.unwrap();
            let resp = self.client.object().copy(
                &obj,
                &bucket,
                &key
            ).await;
            match resp {
                Ok(_) => Ok(
                    GcpBlob{
                        key: Some(key.to_string()),
                        e_tag: Some(obj.etag.clone()),
                        size: Some(obj.size as i64),
                        body: None,
                        content_type,
                        content_range: None,
                        bucket: bucket.to_string(),
                    }
                    ),
                Err(e) => {
                    Err(BlobError::CopyError(
                            String::from(format!("{}",e))
                            ))
                },
            }
        } else {
            return Err(BlobError::CopyError(
                    String::from(r"Format blob_destination_path as {bucket}/{blob_path}")))
        }
    }
    
    async fn write_blob(&self, blob_name: String, content: Option<Bytes>) -> BlobResult<GcpBlob>{
        use bytes::Buf;
        use std::io;

        let mut file: Vec<u8> = Vec::new();
        match content {
            Some(x) => {
                let mut reader = x.reader();
                io::copy(&mut reader, &mut file);
            },
            None => ()
        }
        let resp = self.client.object().create(
            self.name.as_str(),
            file,
            blob_name.as_str(),
            ""
            ).await;
        match resp {
            Ok(obj) => Ok(
                GcpBlob{
                    key: Some(obj.name.to_string()),
                    e_tag: Some(obj.etag.clone()),
                    size: Some(obj.size as i64),
                    body: None,
                    content_type: obj.content_type,
                    content_range: None,
                    bucket: self.name.clone(),
                }
                ),
            Err(e) => {
                Err(BlobError::WriteError(
                        String::from(format!("{}",e))
                        ))
            },
        }
    }

    async fn delete_blob(&self, blob_path: String) -> BlobResult<bool>{
        let resp = self.client.object().delete(
            self.name.as_str(),
            blob_path.as_str(),
            ).await;
        match resp {
            Ok(_) => Ok(true),
            Err(e) => {
                Err(BlobError::DeletionError(
                        String::from(format!("{}",e))
                        ))
            },
        }
    }
}
