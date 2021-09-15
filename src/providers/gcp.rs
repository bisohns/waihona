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
    project: String,
}

impl GcpBlob {
    pub fn new(key: Option<String>, 
               e_tag: Option<String>,
               size: Option<i64>,
               body: Option<Vec<u8>>,
               content_type: Option<String>,
               content_range: Option<String>, 
               bucket: String,
               project: String
               )-> Self {
        GcpBlob{
            key,
            e_tag,
            size,
            body,
            content_type,
            content_range,
            bucket,
            project
        }
    }

    pub async fn get(project_name: &str, bucket: &str, blob_path: &str, content_range: Option<String>) -> BlobResult<Self> {
        let mut buckets = GcpBuckets::new(
            project_name
            );
        let bucket = buckets.open(bucket).await;
        match bucket {
            Ok(b) => {
                b.get_blob(
                    blob_path,
                    content_range
                    ).await
            },
            Err(e) => Err(BlobError::GetError(e.to_string()))
        }
    }
}

#[async_trait]
impl Blob for GcpBlob {
    async fn delete(&self) -> BlobResult<bool> {
        let mut buckets = GcpBuckets::new(&self.project);
        let bucket = buckets.open(&self.bucket).await.unwrap();
        let del = bucket.delete_blob(
            &self.key.as_ref().unwrap(),
            ).await;
        match del {
            Ok(_) => Ok(true),
            Err(e) => {
                Err(BlobError::CopyError(
                        String::from(format!("{}",e))
                        ))
            },
        }
    }
    
    async fn copy(&self,
                  blob_destination_path: &str,
                  content_type: Option<String>
                  ) -> BlobResult<bool> {

        let mut buckets = GcpBuckets::new(&self.project);
        let bucket = buckets.open(&self.bucket).await.unwrap();
        let copied = bucket.copy_blob(
            &self.key.as_ref().unwrap(),
            blob_destination_path,
            content_type
            ).await;
        match copied {
            Ok(_) => Ok(true),
            Err(e) => {
                Err(BlobError::CopyError(
                        String::from(format!("{}",e))
                        ))
            },
        }
    }

    async fn write(&self, content: Option<Bytes>) -> BlobResult<bool> {
        let mut buckets = GcpBuckets::new(&self.project);
        let bucket = buckets.open(&self.bucket).await.unwrap();
        let write = bucket.write_blob(
            &self.key.as_ref().unwrap(),
            content
            ).await;
        match write {
            Ok(_) => Ok(true),
            Err(e) => {
                Err(BlobError::WriteError(
                        String::from(format!("{}",e))
                        ))
            },
        }

    }

    async fn read(&mut self) -> BlobResult<Bytes> {
        let buckets = GcpBuckets::new(&self.project);
        let resp = buckets.client.object().download(
            &self.bucket,
            &self.key.as_ref().unwrap(),
            ).await;
        match resp {
            Ok(res) => Ok(Bytes::from(res)),
            Err(_) => {
                Err(BlobError::ReadError)
            }
    }
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

    async fn open(&mut self, bucket_name: &str) -> BucketResult<GcpBucket>{
        if self.exists(bucket_name).await {
            match self.client.bucket().read(bucket_name).await {
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

    async fn create(&mut self, bucket_name: &str, _location: Option<String>) -> BucketResult<GcpBucket>{
        let new_bucket = NewBucket {
            name: bucket_name.to_string(),
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

    async fn delete(&mut self, bucket_name: &str) -> BucketResult<bool> {
        if self.exists(bucket_name).await {
            let bucket = self.client.bucket().read(bucket_name).await.unwrap();
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
    
    async fn exists(&mut self, bucket_name: &str) -> bool {
        match self.client.bucket().read(bucket_name).await {
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

impl GcpBucket {
    pub async fn exists(project: &str, bucket: &str) -> bool {
        let mut buckets = GcpBuckets::new(project);
        buckets.exists(bucket).await
    }
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
                        GcpBlob::new(
                            Some(obj.name.clone()),
                            Some(obj.etag.clone()),
                            Some(obj.size as i64),
                            None,
                            obj.content_type.clone(),
                            None,
                            self.name.clone(),
                            self.user_project.clone(),
                        ))
                }
                Ok((ret, None))
            },
            Err(_) => Err(BucketError::ListError(String::from("could not list")))
        }
    } async fn get_blob(&self, blob_path: &str, content_range: Option<String>) -> BlobResult<GcpBlob>{
        let resp = self.client.object().read(
            self.name.as_str(), 
            blob_path,
            ).await;
        match resp {
            Ok(k) => {
                Ok(GcpBlob::new(
                    Some(k.name.clone()),
                    Some(k.etag.clone()),
                    Some(k.size as i64),
                    None,
                    k.content_type,
                    content_range,
                    self.name.clone(),
                    self.user_project.clone(),
                ))
            },
            Err(e) => {
                Err(BlobError::GetError(
                    String::from(format!("{}",e))
                    ))
            },
        }
    }

    async fn copy_blob(&self,
                       blob_path: &str, 
                       blob_destination_path: &str,
                       content_type: Option<String>) -> BlobResult<GcpBlob>{
        let re = Regex::new(r"(?P<bucket>.*?)/(?P<blob_path>.*)").unwrap();
        if let Some(captures) = re.captures(blob_destination_path) {
            let bucket = captures
                .name("bucket")
                .unwrap().as_str().to_owned();
            let key = captures
                .name("blob_path")
                .unwrap().as_str().to_owned();
            let obj = self.client.object().read(
                self.name.as_str(), 
                blob_path,
            ).await.unwrap();
            let resp = self.client.object().copy(
                &obj,
                &bucket,
                &key
            ).await;
            match resp {
                Ok(_) => Ok(
                    GcpBlob::new(
                        Some(key.to_string()),
                        Some(obj.etag.clone()),
                        Some(obj.size as i64),
                        None,
                        content_type,
                        None,
                        bucket.to_string(),
                        self.user_project.clone(),
                    )),
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
    
    async fn write_blob(&self, blob_name: &str, content: Option<Bytes>) -> BlobResult<GcpBlob>{
        use bytes::Buf;
        use std::io;

        let mut file: Vec<u8> = Vec::new();
        match content {
            Some(x) => {
                let mut reader = x.reader();
                match io::copy(&mut reader, &mut file){
                    Ok(_) => (),
                    Err(e) => {
                        return Err(BlobError::WriteError(
                                String::from(format!("{}",e))
                                ))
                    }
                }
            },
            None => ()
        }
        let resp = self.client.object().create(
            self.name.as_str(),
            file,
            blob_name,
            ""
            ).await;
        match resp {
            Ok(obj) => Ok(
                GcpBlob::new(
                    Some(obj.name.to_string()),
                    Some(obj.etag.clone()),
                    Some(obj.size as i64),
                    None,
                    obj.content_type,
                    None,
                    self.name.clone(),
                    self.user_project.clone(),
                )),
            Err(e) => {
                Err(BlobError::WriteError(
                        String::from(format!("{}",e))
                        ))
            },
        }
    }

    async fn delete_blob(&self, blob_path: &str) -> BlobResult<bool>{
        let resp = self.client.object().delete(
            self.name.as_str(),
            blob_path,
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
