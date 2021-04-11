use async_trait::async_trait;
use crate::types::bucket::{Buckets, Bucket};
//use crate::types::blob::{Blob};
use rusoto_core::{Region};
use rusoto_s3::{S3, S3Client};

pub struct AwsBuckets{
    s3: S3Client,
}

#[derive(Debug)]
pub struct AwsBucket{
    name: String,
}

impl AwsBucket {
    fn new(name: String) -> Self {
        AwsBucket {
            name
        }

    }
}

#[derive(Debug)]
pub struct AwsBlob;


impl AwsBuckets {
    fn new(region: Region) -> Self {
        AwsBuckets {
            s3: S3Client::new(region)
        }

    }

}

impl Bucket for AwsBucket {
}

#[async_trait]
impl Buckets for AwsBuckets {
    async fn list(&self) -> Box<Vec<AwsBucket>> {
        let resp = self.s3.list_buckets().await.unwrap();
        let mut buckets: Vec<AwsBucket> = Vec::new();
        for bucket in resp.buckets.unwrap().iter() {
            if bucket.name.is_some(){
                let bucket_found = AwsBucket::new(
                    String::from(bucket.name.clone().unwrap())
                    );
                buckets.push(bucket_found);
            }
        }
        Box::new(buckets)
    }

}
