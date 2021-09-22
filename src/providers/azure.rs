use crate::types::errors::{BlobError, BlobResult, BucketError, BucketResult};
use azure_core::prelude::*;
use azure_storage::blob::prelude::*;
use azure_storage::core::prelude::*;

#[derive(Debug)]
pub struct AzureBuckets {
    pub client: std::sync::Arc<StorageClient>,
    pub storage_account: String,
}

#[derive(Debug)]
pub struct AzureBucket {
    pub name: String,
    pub client: std::sync::Arc<ContainerClient>,
    pub storage_account: String,
    pub e_tag: String,
}

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

impl AzureBucket {
    pub async fn exists(storage_account: &str, bucket: &str) -> bool {
        let mut buckets = AzureBuckets::new(storage_account.to_owned());
        buckets.exists(bucket).await
    }

    pub async fn list_blobs(&self, marker: Option<String>) -> Vec<AzureBlob> {
        let next_marker = NextMarker::from_possibly_empty_string(marker);
        let response;
        match next_marker {
            Some(marker) => {
                response = self.client.list_blobs().next_marker(marker).execute().await;
            }
            None => {
                response = self.client.list_blobs().execute().await;
            }
        }

        let mut blobs: Vec<AzureBlob> = Vec::new();
        for blob in response.unwrap().blobs.blobs.iter() {
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
        blobs
    }
}

impl AzureBuckets {
    pub fn new(storage_account: String) -> AzureBuckets {
        let key = std::env::var("AZURE_SECRET_ACCESS_KEY")
            .expect("Set env variable AZURE_SECRET_ACCESS_KEY");
        let http_client = new_http_client();
        AzureBuckets {
            client: StorageAccountClient::new_access_key(
                http_client.clone(),
                &storage_account,
                &key,
            )
            .as_storage_client(),
            storage_account: storage_account.clone(),
        }
    }

    pub async fn list_containers(&mut self) -> Vec<AzureBucket> {
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
                e_tag: bucket.e_tag.clone(),
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

    pub async fn open(&mut self, bucket_name: &str) -> BucketResult<AzureBucket> {
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
                e_tag: container.e_tag.clone(),
            }),
            None => Err(BucketError::NotFound),
        }
    }
}
