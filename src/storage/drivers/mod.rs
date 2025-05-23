use std::path::Path;

use async_trait::async_trait;
use bytes::Bytes;
use opendal::Reader;

#[cfg(feature = "storage_aws_s3")]
pub mod aws;
#[cfg(feature = "storage_azure")]
pub mod azure;
#[cfg(feature = "storage_gcp")]
pub mod gcp;
pub mod local;
pub mod mem;
pub mod null;
pub mod opendal_adapter;

use super::StorageResult;

#[derive(Debug)]
pub struct UploadResponse {
    pub e_tag: Option<String>,
    pub version: Option<String>,
}

/// TODO: Add more methods to `GetResponse` to read the content in different
/// ways
///
/// For example, we can read a specific range of bytes from the stream.
pub struct GetResponse {
    stream: Reader,
}

impl GetResponse {
    pub(crate) fn new(stream: Reader) -> Self {
        Self { stream }
    }

    /// Read all content from the stream and return as `Bytes`.
    ///
    /// # Errors
    ///
    /// Returns a `StorageError` with the reason for the failure.
    pub async fn bytes(&self) -> StorageResult<Bytes> {
        Ok(self.stream.read(..).await?.to_bytes())
    }
}

#[async_trait]
pub trait StoreDriver: Sync + Send {
    /// Uploads the content represented by `Bytes` to the specified path in the
    /// object store.
    ///
    /// # Errors
    ///
    /// Returns a `StorageResult` with the result of the upload operation.
    async fn upload(&self, path: &Path, content: &Bytes) -> StorageResult<UploadResponse>;

    /// Retrieves the content from the specified path in the object store.
    ///
    /// # Errors
    ///
    /// Returns a `StorageResult` with the result of the retrieval operation.
    async fn get(&self, path: &Path) -> StorageResult<GetResponse>;

    /// Deletes the content at the specified path in the object store.
    ///
    /// # Errors
    ///
    /// Returns a `StorageResult` indicating the success of the deletion
    /// operation.
    async fn delete(&self, path: &Path) -> StorageResult<()>;

    /// Renames or moves the content from one path to another in the object
    /// store.
    ///
    /// # Errors
    ///
    /// Returns a `StorageResult` indicating the success of the rename/move
    /// operation.
    async fn rename(&self, from: &Path, to: &Path) -> StorageResult<()>;

    /// Copies the content from one path to another in the object store.
    ///
    /// # Errors
    ///
    /// Returns a `StorageResult` indicating the success of the copy operation.
    async fn copy(&self, from: &Path, to: &Path) -> StorageResult<()>;

    /// Checks if the content exists at the specified path in the object store.
    ///
    /// # Errors
    ///
    /// Returns a `StorageResult` with a boolean indicating the existence of the
    /// content.
    async fn exists(&self, path: &Path) -> StorageResult<bool>;
}
