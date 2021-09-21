#[tokio::test]
#[cfg(feature = "aws")]
async fn test_bucket_exists() {
    use crate::providers;
    let resp = providers::aws::AwsBucket::exists("us-east-2", "waihona").await;
    assert!(resp);
}

#[tokio::test]
#[cfg(feature = "aws")]
async fn test_bucket_open() {
    use crate::providers;
    use crate::types::bucket::{Bucket, Buckets};
    let mut aws_buckets = providers::aws::AwsBuckets::new("us-east-2");
    let waihona = aws_buckets.open("waihona").await.unwrap();
    let blobs = waihona.list_blobs(None).await;
    println!("{:?}", blobs);
}

#[tokio::test]
#[cfg(feature = "aws")]
async fn test_get_blob() {
    use crate::providers;
    use crate::types::blob::Blob;
    use crate::types::bucket::{Bucket, Buckets};
    let mut aws_buckets = providers::aws::AwsBuckets::new("us-east-2");
    let waihona = aws_buckets.open("waihona").await.unwrap();
    let mut blob = waihona.get_blob("reka-store.txt", None).await.unwrap();

    let res = blob.read().await.unwrap();
    let res_str = std::str::from_utf8(&res);
    println!("{:?}", res_str);
    //    // write data to blob
    //    let res_write = waihona.write_blob(
    //        "copy-reka.txt".to_owned(),
    //        Some(Bytes::from("Hello world"))
    //        ).await
    //        .unwrap();
    //    let cp_blob = waihona.copy_blob(
    //        "reka-store.txt".to_owned(),
    //        "waihona/copy-reka.txt".to_owned(),
    //        None
    //        ).await
    //        .unwrap();
    //    println!("{:?}", cp_blob);
    //    cp_blob.copy(
    //        "waihona/sec-copy.txt",
    //        None
    //        ).await;
    //    let res = blob.delete().await;
    //    println!("{:?}", res);
}
