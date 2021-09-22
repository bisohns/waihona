use crate::types::blob::Blob;
use crate::types::bucket::{Bucket, Buckets};
use crate::types::errors::{BlobError, BlobResult, BucketError, BucketResult};
use async_trait::async_trait;
use azure_core::prelude::*;
use azure_storage::blob::prelude::*;
use azure_storage::core::prelude::*;
use bytes::Bytes;
use futures::stream::StreamExt;
use regex::Regex;
use std::time::Duration;

#[derive(Debug)]
pub struct AzureBlob {
    key: String,
    e_tag: Etag,
    body: Option<Vec<u8>>,
    content_type: String,
    content_length: u64,
    container: String,
    storage_account: String,
}

impl AzureBlob {
    pub fn new(
        key: String,
        e_tag: Etag,
        body: Option<Vec<u8>>,
        content_type: String,
        content_length: u64,
        container: String,
        storage_account: String,
    ) -> Self {
        AzureBlob {
            key,
            e_tag,
            body,
            content_type,
            content_length,
            container,
            storage_account,
        }
    }
    pub async fn get(
        storage_account: &str,
        container: &str,
        blob_name: &str,
        content_range: Option<String>,
    ) -> BlobResult<Self> {
        let mut buckets = AzureBuckets::new(storage_account.to_owned());
        let bucket = buckets.open(container).await;
        match bucket {
            Ok(b) => b.get_blob(blob_name, content_range).await,
            Err(e) => Err(BlobError::GetError(e.to_string())),
        }
    }
}

#[async_trait]
impl Blob for AzureBlob {
    async fn copy(
        &self,
        blob_destination_path: &str,
        content_type: Option<String>,
    ) -> BlobResult<bool> {
        let mut buckets = AzureBuckets::new(self.storage_account.to_owned());
        let bucket = buckets.open(&self.container).await.unwrap();
        let copied = bucket
            .copy_blob(&self.key, blob_destination_path, content_type)
            .await;
        match copied {
            Ok(_) => Ok(true),
            Err(e) => Err(BlobError::CopyError(String::from(format!("{}", e)))),
        }
    }

    async fn write(&self, content: Option<Bytes>) -> BlobResult<bool> {
        let mut buckets = AzureBuckets::new(self.storage_account.to_owned());
        let bucket = buckets.open(&self.container).await.unwrap();
        let write = bucket.write_blob(&self.key, content).await;
        match write {
            Ok(_) => Ok(true),
            Err(e) => {
                Err(BlobError::WriteError(String::from(format!("{}", e))))
            }
        }
    }
    async fn read(&mut self) -> BlobResult<Bytes> {
        let buckets = AzureBuckets::new(self.storage_account.to_owned());
        let blob_client = buckets
            .client
            .as_container_client(&self.container)
            .as_blob_client(&self.key);

        let mut complete_response = Vec::new();

        let mut stream = Box::pin(blob_client.get().stream(1024 * 8));
        while let Some(value) = stream.next().await {
            let data = value.unwrap().data;
            complete_response.extend(&data as &[u8]);
        }
        Ok(Bytes::from(complete_response))
    }

    async fn delete(&self) -> BlobResult<bool> {
        let mut buckets = AzureBuckets::new(self.storage_account.to_owned());
        let bucket = buckets.open(&self.container).await.unwrap();
        let del = bucket.delete_blob(&self.key).await;
        match del {
            Ok(_) => Ok(true),
            Err(e) => {
                Err(BlobError::DeletionError(String::from(format!("{}", e))))
            }
        }
    }
}

#[derive(Debug)]
pub struct AzureBucket {
    pub name: String,
    pub client: std::sync::Arc<ContainerClient>,
    pub storage_account: String,
}

impl AzureBucket {
    pub async fn exists(storage_account: &str, bucket: &str) -> bool {
        let mut buckets = AzureBuckets::new(storage_account.to_owned());
        buckets.exists(bucket).await
    }
}

#[async_trait]
impl Bucket<AzureBlob> for AzureBucket {
    async fn get_blob(
        &self,
        blob_path: &str,
        content_range: Option<String>,
    ) -> BlobResult<AzureBlob> {
        let resp = self.client.as_blob_client(blob_path).get().execute().await;
        match resp {
            Ok(k) => Ok(AzureBlob::new(
                k.blob.name.to_owned(),
                k.blob.properties.etag.to_owned(),
                None,
                k.blob.properties.content_type.to_owned(),
                k.blob.properties.content_length,
                self.name.clone(),
                self.storage_account.clone(),
            )),
            Err(e) => Err(BlobError::GetError(String::from(format!("{}", e)))),
        }
    }

    async fn copy_blob(
        &self,
        blob_path: &str,
        blob_destination_path: &str,
        content_type: Option<String>,
    ) -> BlobResult<AzureBlob> {
        let re = Regex::new(r"(?P<bucket>.*?)/(?P<blob_path>.*)").unwrap();
        if let Some(captures) = re.captures(blob_destination_path) {
            let bucket = captures.name("bucket").unwrap().as_str().to_owned();
            let key = captures.name("blob_path").unwrap().as_str().to_owned();
            let absolute_path;
            if bucket == self.name {
                absolute_path = key;
            } else {
                absolute_path = format!("{}/{}", bucket, key).to_owned();
            }
            let buckets = AzureBuckets::new(self.storage_account.to_owned());
            let source_url = format!(
                "{}{}/{}",
                buckets.account_client.blob_storage_url().as_str(),
                self.name,
                blob_path
            );
            let blob = self.client.as_blob_client(&absolute_path);

            let response = blob
                .copy_from_url(&source_url)
                .is_synchronous(true)
                .execute()
                .await;
            match response {
                Ok(_) => {
                    let blob =
                        self.get_blob(absolute_path.as_str(), None).await;
                    match blob {
                        Ok(blob) => Ok(blob),
                        Err(_) => Err(BlobError::NotFound),
                    }
                }
                Err(e) => {
                    Err(BlobError::GetError(String::from(format!("{}", e))))
                }
            }
        } else {
            return Err(BlobError::CopyError(String::from(
                r"Format blob_destination_path as {bucket}/{blob_path}",
            )));
        }
    }

    async fn list_blobs(
        &self,
        marker: Option<String>,
    ) -> BucketResult<(Vec<AzureBlob>, Option<String>)> {
        let next_marker = NextMarker::from_possibly_empty_string(marker);
        let response;
        match next_marker {
            Some(marker) => {
                response = self
                    .client
                    .list_blobs()
                    .next_marker(marker)
                    .execute()
                    .await;
            }
            None => {
                response = self.client.list_blobs().execute().await;
            }
        }

        let mut res = response.unwrap();
        let mut blobs: Vec<AzureBlob> = Vec::new();
        for blob in &mut res.blobs.blobs.iter() {
            let found_blob = AzureBlob {
                key: blob.name.to_owned(),
                e_tag: blob.properties.etag.to_owned(),
                body: None,
                content_type: blob.properties.content_type.to_owned(),
                content_length: blob.properties.content_length,
                container: self.name.to_owned(),
                storage_account: self.storage_account.to_owned(),
            };
            blobs.push(found_blob);
        }
        let nex_marker;
        match &mut res.next_marker {
            Some(marker) => nex_marker = Some(marker.as_str().to_owned()),
            None => nex_marker = None,
        };
        Ok((blobs, nex_marker))
    }

    async fn delete_blob(&self, blob_path: &str) -> BlobResult<bool> {
        let resp = self
            .client
            .as_blob_client(blob_path)
            .delete()
            .execute()
            .await;
        match resp {
            Ok(_) => Ok(true),
            Err(e) => {
                Err(BlobError::DeletionError(String::from(format!("{}", e))))
            }
        }
    }

    async fn write_blob(
        &self,
        blob_name: &str,
        content: Option<Bytes>,
    ) -> BlobResult<AzureBlob> {
        use bytes::Buf;
        use std::io;

        let mut file: Vec<u8> = Vec::new();
        match content {
            Some(x) => {
                let mut reader = x.reader();
                match io::copy(&mut reader, &mut file) {
                    Ok(_) => (),
                    Err(e) => {
                        return Err(BlobError::WriteError(String::from(
                            format!("{}", e),
                        )))
                    }
                }
            }
            None => (),
        }
        let resp = self
            .client
            .as_blob_client(blob_name)
            .put_block_blob(file.clone())
            .content_type("")
            .execute()
            .await;
        match resp {
            Ok(_) => {
                let blob = self.get_blob(blob_name, None).await;
                match blob {
                    Ok(blob) => Ok(blob),
                    Err(_) => Err(BlobError::NotFound),
                }
            }
            Err(e) => {
                Err(BlobError::WriteError(String::from(format!("{}", e))))
            }
        }
    }
}

#[derive(Debug)]
pub struct AzureBuckets {
    pub client: std::sync::Arc<StorageClient>,
    pub account_client: std::sync::Arc<StorageAccountClient>,
    pub storage_account: String,
}

impl AzureBuckets {
    pub fn new(storage_account: String) -> AzureBuckets {
        let key = std::env::var("AZURE_SECRET_ACCESS_KEY")
            .expect("Set env variable AZURE_SECRET_ACCESS_KEY");
        let http_client = new_http_client();
        let storage_account_client = StorageAccountClient::new_access_key(
            http_client.clone(),
            &storage_account,
            &key,
        );
        AzureBuckets {
            client: storage_account_client.as_storage_client(),
            account_client: storage_account_client,
            storage_account: storage_account,
        }
    }
}

#[async_trait]
impl Buckets<AzureBucket, AzureBlob> for AzureBuckets {
    async fn list(&mut self) -> Vec<AzureBucket> {
        let response = self
            .client
            .list_containers()
            .include_metadata(true)
            .execute()
            .await;
        let mut buckets: Vec<AzureBucket> = Vec::new();
        for bucket in response.unwrap().incomplete_vector.iter() {
            let bucket_found = AzureBucket {
                name: bucket.name.clone(),
                client: self.client.as_container_client(&bucket.name),
                storage_account: self.storage_account.clone(),
            };
            buckets.push(bucket_found);
        }
        buckets
    }

    async fn exists(&mut self, bucket_name: &str) -> bool {
        let containers = self.client.list_containers().execute().await;
        containers
            .unwrap()
            .incomplete_vector
            .iter()
            .find(|item| item.name == bucket_name)
            .is_some()
    }

    async fn create(
        &mut self,
        bucket_name: &str,
        _location: Option<String>,
    ) -> BucketResult<AzureBucket> {
        match self
            .client
            .as_container_client(bucket_name)
            .create()
            .public_access(PublicAccess::None)
            .timeout(Duration::from_secs(100))
            .execute()
            .await
        {
            Ok(_) => Ok(AzureBucket {
                name: bucket_name.to_owned(),
                client: self.client.as_container_client(bucket_name),
                storage_account: self.storage_account.clone(),
            }),
            Err(e) => {
                Err(BucketError::CreationError(String::from(format!("{}", e))))
            }
        }
    }

    async fn delete(&mut self, bucket_name: &str) -> BucketResult<bool> {
        if self.exists(bucket_name).await {
            match self
                .client
                .as_container_client(bucket_name)
                .delete()
                .execute()
                .await
            {
                Ok(_) => Ok(true),
                Err(e) => Err(BucketError::DeletionError(String::from(
                    format!("{}", e),
                ))),
            }
        } else {
            Ok(false)
        }
    }

    async fn open(&mut self, bucket_name: &str) -> BucketResult<AzureBucket> {
        let response = self.client.list_containers().execute().await;
        match response
            .unwrap()
            .incomplete_vector
            .iter()
            .find(|item| item.name == bucket_name)
        {
            Some(container) => Ok(AzureBucket {
                name: container.name.clone(),
                client: self.client.as_container_client(&container.name),
                storage_account: self.storage_account.clone(),
            }),
            None => Err(BucketError::NotFound),
        }
    }
}
