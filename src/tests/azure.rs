#[tokio::test]
#[cfg(feature = "azure")]
async fn test_container_listing() {
    use crate::providers;
    // use crate::types::bucket::Buckets;
    let mut gcp_buckets = providers::azure::AzureBuckets::new("waihona".to_owned());
    let resp = gcp_buckets.list_containers().await;
    println!("{:?}", resp);
}

#[tokio::test]
#[cfg(feature = "azure")]
async fn test_container_exists() {
    use crate::providers;
    let resp = providers::azure::AzureBucket::exists(
        "waihona",     // valid storage account
        "fake-bucket", // fake container name
    )
    .await;
    assert!(resp == false);
}

#[tokio::test]
#[cfg(feature = "azure")]
async fn test_container_list_blobs() {
    use crate::providers;
    use crate::types::bucket::{Bucket, Buckets};
    let mut azure_buckets = providers::azure::AzureBuckets::new("waihona".to_owned());
    let resp = azure_buckets.open("waihona").await;
    let waihona = resp.unwrap();
    let blobs = waihona.list_blobs(None).await;
    println!("{:?}", blobs);
}
