#[tokio::test]
#[cfg(feature = "azure")]
async fn test_container_listing() {
    use crate::providers;
    use crate::types::bucket::Buckets;
    let mut gcp_buckets =
        providers::azure::AzureBuckets::new("waihona".to_owned());
    let resp = gcp_buckets.list().await;
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
    let mut azure_buckets =
        providers::azure::AzureBuckets::new("waihona".to_owned());
    let resp = azure_buckets.open("waihona").await;
    let waihona = resp.unwrap();
    let blobs = waihona.list_blobs(None).await;
    println!("{:?}", blobs);
}

#[tokio::test]
#[cfg(feature = "azure")]
async fn test_container_read_blob() {
    use crate::providers;
    use crate::types::blob::Blob;
    use crate::types::bucket::{Bucket, Buckets};
    use bytes::Bytes;
    let mut azure_buckets =
        providers::azure::AzureBuckets::new("waihona".to_owned());
    let resp = azure_buckets.open("waihona").await;
    let waihona = resp.unwrap();

    let mut blob = providers::azure::AzureBlob::get(
        "waihona",
        "waihona",
        "CV latest.pdf",
        None,
    )
    .await
    .unwrap();
    println!("Get Single Blob: {:?}", blob);
    let read = blob.read().await.unwrap();
    println!("Reading Blob: {:?}", read);
    let copied = waihona
        .copy_blob("CV latest.pdf", "waihona/copied.pdf", None)
        .await
        .unwrap();
    println!("copied {:?}", copied);
    let del = copied.delete().await.unwrap();
    assert!(del);
    let content = Some(Bytes::from(r"{'example': 1}"));
    let mut new = waihona.write_blob("new.json", content).await.unwrap();
    let read = new.read().await.unwrap();
    assert!(read.eq(&Bytes::from(r"{'example': 1}")));
}
