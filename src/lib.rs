pub mod types;
pub mod providers;

//pub fn get_provider_bucket<T>(provider: String) -> Box<T>
//    where T: types::bucket::Bucket {

//    }

//pub fn get_provider_blob<T>(provider: String) -> Box<T>
//    where T: types::blob::Blob {

//    }

#[tokio::test]
async fn test_bucket_exists() {
    use crate::types::bucket::{Buckets};
    use rusoto_core::{Region};
    let aws_buckets = providers::aws::AwsBuckets::new(
        Region::UsEast2
        );
    let resp = aws_buckets.exists(
        String::from("waihona")
        ).await;
    assert!(resp);
}

#[tokio::test]
async fn test_bucket_open() {
    use crate::types::bucket::{Buckets, Bucket};
    use rusoto_core::{Region};
    let aws_buckets = providers::aws::AwsBuckets::new(
        Region::UsEast2
        );
    let waihona = aws_buckets.open(
        String::from("waihona"),
        ).await.unwrap();
    let blobs = waihona.list_blobs(None).await;
    println!("{:?}", blobs);
}

#[tokio::test]
async fn test_get_blob() {
    use crate::types::bucket::{Buckets, Bucket};
    use crate::types::blob::{Blob};
    use rusoto_core::{Region};
    let aws_buckets = providers::aws::AwsBuckets::new(
        Region::UsEast2
        );
    let waihona = aws_buckets.open(
        String::from("waihona"),
        ).await.unwrap();
    let mut blob = waihona.get_blob(
        "reka-store.txt".to_owned(),
        None
        ).await
        .unwrap();
    let res = blob.read().await.unwrap();
    let res_str = std::str::from_utf8(&res);
    println!("{:?}", res_str);
//    let cp_blob = waihona.copy_blob(
//        "reka-store.txt".to_owned(),
//        "waihona/copy-reka.txt".to_owned(),
//        None
//        ).await
//        .unwrap();
//    println!("{:?}", cp_blob);
//    cp_blob.copy(
//        "waihona/sec-copy.txt".to_owned(),
//        None
//        ).await;
//    let res = blob.delete().await;
//    println!("{:?}", res);
}
