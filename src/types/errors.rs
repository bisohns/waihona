use strum_macros::Display;
/// Bucket based errors
#[derive(Debug, Display)]
pub enum BucketError {
    /// Bucket specified was not found
    NotFound,
    /// Bucket error during creation
    CreationError(String),
    /// Bucket deletion error
    DeletionError(String),
    /// Bucket listing error
    ListError(String),
}

/// Blob based errors
#[derive(Debug, Display)]
pub enum BlobError {
    /// Blob specified was not found
    NotFound,
    /// Blob could not be gotten
    GetError(String),
    /// Blob could not be read
    ReadError,
    /// Could not delete blob
    DeletionError(String),
    /// Could not copy blob
    CopyError(String),
    /// Could not write blob
    WriteError(String),
}

/// Provider based errors
#[derive(Debug)]
pub enum ProviderError {
    /// Provider specified was not found
    NotFound,
}

/// Bucket Result type
pub type BucketResult<T> = std::result::Result<T, BucketError>;
/// Blob Result type
pub type BlobResult<T> = std::result::Result<T, BlobError>;
