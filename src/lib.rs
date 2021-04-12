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
    use crate::types::bucket::{Buckets, Bucket};
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
