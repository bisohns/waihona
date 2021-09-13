
#[tokio::test]
#[cfg(feature = "gcp")]
async fn test_bucket_listing() {
    use crate::types::bucket::{Buckets};
    use crate::providers;
    let mut gcp_buckets = providers::gcp::GcpBuckets::new(
        "psyched-myth-306812"
        );
    let resp = gcp_buckets.list().await;
    println!("{:?}", resp[0].name);
}

#[tokio::test]
#[cfg(feature = "gcp")]
async fn test_bucket_list_blobs() {
    use crate::types::bucket::{Buckets, Bucket};
    use crate::providers;
    let mut gcp_buckets = providers::gcp::GcpBuckets::new(
        "psyched-myth-306812"
        );
    let resp =  gcp_buckets.open(
        "mythra".to_owned()
        ).await;
    let mythra = resp.unwrap();
    let blobs = mythra.list_blobs(None).await;
    println!("{:?}", blobs);
}

#[tokio::test]
#[cfg(feature = "gcp")]
async fn test_bucket_get_blob() {
    use crate::types::bucket::{Buckets, Bucket};
    use crate::providers;
    let mut gcp_buckets = providers::gcp::GcpBuckets::new(
        "psyched-myth-306812"
        );
    let resp =  gcp_buckets.open(
        "mythra".to_owned()
        ).await;
    let mythra = resp.unwrap();
    let blob = mythra.get_blob(
        "Screenshot from 2021-03-24 20-47-02.png".to_owned(),
        None
        ).await;
    println!("{:?}", blob);
    let copied = mythra.copy_blob(
        "Screenshot from 2021-03-24 20-47-02.png".to_owned(),
        "mythra/copied.png".to_owned(),
        None
        ).await;
    println!("{:?}", copied);
    let del = mythra.delete_blob(
        "copied.png".to_owned()
        ).await.unwrap();
    println!("{:?}", del);
    use bytes::Bytes;
    let new = mythra.write_blob(
        "new.json".to_owned(),
        Some(Bytes::from(r"{'example': 1}"))
        ).await;
    println!("{:?}", new);
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
