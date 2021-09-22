#[tokio::test]
#[cfg(feature = "gcp")]
async fn test_bucket_listing() {
    use crate::providers;
    use crate::types::bucket::Buckets;
    let mut gcp_buckets =
        providers::gcp::GcpBuckets::new("psyched-myth-306812");
    let resp = gcp_buckets.list().await;
    println!("{:?}", resp[0].name);
}

#[tokio::test]
#[cfg(feature = "gcp")]
async fn test_bucket_exists() {
    use crate::providers;
    let resp = providers::gcp::GcpBucket::exists(
        "psyched-myth-306812", // valid project name
        "fake-bucket",         // fake bucket name
    )
    .await;
    assert!(resp == false);
}

#[tokio::test]
#[cfg(feature = "gcp")]
async fn test_bucket_list_blobs() {
    use crate::providers;
    use crate::types::bucket::{Bucket, Buckets};
    let mut gcp_buckets =
        providers::gcp::GcpBuckets::new("psyched-myth-306812");
    let resp = gcp_buckets.open("mythra").await;
    let mythra = resp.unwrap();
    let blobs = mythra.list_blobs(None).await;
    println!("{:?}", blobs);
}

#[tokio::test]
#[cfg(feature = "gcp")]
async fn test_bucket_get_blob() {
    use crate::providers;
    use crate::types::blob::Blob;
    use crate::types::bucket::{Bucket, Buckets};
    use bytes::Bytes;
    let mut gcp_buckets =
        providers::gcp::GcpBuckets::new("psyched-myth-306812");
    let resp = gcp_buckets.open("mythra").await;
    let mythra = resp.unwrap();
    let blob = providers::gcp::GcpBlob::get(
        "pysched-myth-306812",
        "mythra",
        "Screenshot from 2021-03-24 20-47-02.png",
        None,
    )
    .await
    .unwrap();
    println!("{:?}", blob);
    let copied = mythra
        .copy_blob(
            "Screenshot from 2021-03-24 20-47-02.png",
            "mythra/copied.png",
            Some("image/png".to_owned()),
        )
        .await
        .unwrap();
    println!("{:?}", copied);
    let del = copied.delete().await.unwrap();
    assert!(del);
    let content = Some(Bytes::from(r"{'example': 1}"));
    let mut new = mythra.write_blob("new.json", content).await.unwrap();
    let read = new.read().await.unwrap();
    assert!(read.eq(&Bytes::from(r"{'example': 1}")));
}

//#[tokio::test]
//#[cfg(feature = "gcp")]
//async fn test_bucket_creation() {
//    use crate::types::bucket::{Buckets};
//    use crate::providers;
//    let mut gcp_buckets = providers::gcp::GcpBuckets::new(
//        "psyched-myth-306812"
//        ).await;
//    let resp = gcp_buckets.create(
//        "mythra-new".to_owned(),
//        None
//        ).await.unwrap();
//    println!("{:?}", resp.name);
//}

//#[tokio::test]
//#[cfg(feature = "gcp")]
//async fn test_bucket_deletion() {
//    use crate::types::bucket::{Buckets};
//    use crate::providers;
//    let mut gcp_buckets = providers::gcp::GcpBuckets::new(
//        "psyched-myth-306812"
//        ).await;
//    let resp = gcp_buckets.delete(
//        "mythra-new".to_owned(),
//        ).await.unwrap();
//    assert!(resp);
//}
