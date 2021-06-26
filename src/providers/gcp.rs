use std::env;
use hyper::body::Body;
use async_trait::async_trait;
use bytes::Bytes;
use crate::types::bucket::{Buckets, Bucket};
use crate::types::blob::{Blob};
use crate::types::errors::{
    BucketResult, BucketError, BlobResult,BlobError
};
use google_cloud::storage::{
    Bucket as StorageBucket, Client
};
use yup_oauth2::{ServiceAccountAuthenticator, InstalledFlowReturnMethod};
use google_storage1::api::Storage;

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
    pub user_project: String,
}

/// project name on GCP
/// Will make use of the exported credential at 
/// GOOGLE_APPLICATION_CREDENTIALS
impl GcpBuckets {
    pub async fn new(project_name: impl Into<String>) -> GcpBuckets {
        let name = project_name.into();
        GcpBuckets{
            client: Client::new(name.clone()).await.unwrap(),
            user_project: name
        }
    }

}

#[derive(Debug)]
pub struct GcpBlob {
    key: Option<String>,
    e_tag: Option<String>,
    size: Option<i64>,
    body: Option<Body>,
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
        let resp = self.client.buckets().await;
        let mut buckets: Vec<GcpBucket> = Vec::new();
        for bucket in resp.unwrap().iter() {
            let bucket_found = GcpBucket::new(
                String::from(bucket.name()),
                Some(bucket.clone()),
                self.user_project.clone()
                );
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
                    Some(bucket),
                    self.user_project.clone()
                    )
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
                        Some(a),
                        self.user_project.clone()
                        ))
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
                let _ = b.delete().await;
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
    pub bucket: Option<StorageBucket>,
    pub user_project: String,
}

#[async_trait]
impl Bucket<GcpBlob> for GcpBucket {

    async fn list_blobs(&self, marker: Option<String>) -> BucketResult<(Vec<GcpBlob>, Option<String>)>{
        match GcpBucket::hub().await {
            Err(e) => {
                Err(BucketError::ListError(e.to_string()))
            },
            Ok(hub) => {
                let mut ret: Vec<GcpBlob> = Vec::new();
                let project: &str  = &self.user_project.clone();
                let bucket: &str = &self.name.clone();
                let page_token = match marker {
                    None => String::from(""),
                    Some(a) => a,
                };
                match hub.objects()
                    .list(bucket)
                    .user_project(project)
                    .page_token(&page_token)
                    .doit().await {

                        Err(e)=>Err(BucketError::ListError(e.to_string())),
                        Ok(body) => {
                            let (_, objects) = body;
                            for item in objects.items.unwrap() {
                                let size = match item.size {
                                    None => None,
                                    Some(a) => Some(
                                        a.as_str().parse::<i64>().unwrap()
                                        )
                                };

                                ret.push(
                                    GcpBlob{
                                        key: None,
                                        e_tag: item.etag,
                                        size,
                                        body: None,
                                        content_type: item.content_type,
                                        content_range: None,
                                        bucket: self.name.clone(),
                                    })
                            }
                            Ok((ret, objects.next_page_token))

                        }
                    }
            }
        }
    }

    async fn get_blob(&self, blob_path: String, content_range: Option<String>) -> BlobResult<GcpBlob>{
        unimplemented!();
    }

    async fn copy_blob(&self,
                       blob_path: String, 
                       blob_destination_path: String,
                       content_type: Option<String>) -> BlobResult<GcpBlob>{
        unimplemented!();
    }
    
    async fn write_blob(&self, blob_name: String, content: Option<Bytes>) -> BlobResult<GcpBlob>{
        unimplemented!();
    }

    async fn delete_blob(&self, blob_path: String) -> BlobResult<bool>{
        unimplemented!();
    }
}

impl GcpBucket {
    pub fn new(name: String, bucket: Option<StorageBucket>, user_project: String) -> GcpBucket {
        GcpBucket {
            name,
            bucket,
            user_project
        }
    }

    /// Create a Storage object for making object calls to gcp
    pub async fn hub() -> BucketResult<Storage> {
        match env::var("GOOGLE_APPLICATION_CREDENTIALS") {
            Ok(b) => {
                let secret = yup_oauth2::read_service_account_key(b)
                    .await.expect("client secret");
                let auth = ServiceAccountAuthenticator::builder(
                    secret,
                    ).build().await.unwrap();
                let hub = Storage::new(hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()), auth);
                Ok(hub)
            },
            Err(e) => Err(
                BucketError::CredError(
                    String::from("GOOGLE_APPLICATION_CREDENTIALS not found")
                    )
                )
        }
    }

}

