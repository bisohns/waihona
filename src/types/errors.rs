/// Bucket based errors
pub enum BucketError {
    /// Bucket specified was not found
    NotFound,
}

/// Blob based errors
pub enum BlobError {
    /// Blob specified was not found
    NotFound,
}

/// Provider based errors
pub enum ProviderError {
    /// Provider specified was not found
    NotFound,
}

/// Bucket Result type
pub type BucketResult<T> = std::result::Result<T, BucketError>;
/// Blob Result type
pub type BlobResult<T> = std::result::Result<T, BlobError>;
