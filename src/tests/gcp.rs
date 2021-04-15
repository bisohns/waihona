
#[tokio::test]
#[cfg(feature = "gcp")]
async fn test_bucket_listing() {
    use crate::types::bucket::{Buckets};
    use crate::providers;
    let mut gcp_buckets = providers::gcp::GcpBuckets::new(
        "psyched-myth-306812"
        ).await;
    let resp = gcp_buckets.list().await;
    println!("{:?}", resp[0].name);
}
