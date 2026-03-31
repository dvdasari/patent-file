use anyhow::Result;
use std::sync::Arc;

#[async_trait::async_trait]
pub trait StorageClient: Send + Sync {
    async fn upload(&self, key: &str, data: &[u8], content_type: &str) -> Result<()>;
    async fn download_url(&self, key: &str, expires_secs: u64) -> Result<String>;
    async fn delete(&self, key: &str) -> Result<()>;
}

pub struct LocalStorageClient {
    base_path: String,
    port: u16,
}

impl LocalStorageClient {
    pub fn new(base_path: &str, port: u16) -> Self {
        std::fs::create_dir_all(base_path).ok();
        Self {
            base_path: base_path.to_string(),
            port,
        }
    }
}

#[async_trait::async_trait]
impl StorageClient for LocalStorageClient {
    async fn upload(&self, key: &str, data: &[u8], _content_type: &str) -> Result<()> {
        let path = format!("{}/{}", self.base_path, key);
        if let Some(parent) = std::path::Path::new(&path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, data)?;
        Ok(())
    }

    async fn download_url(&self, key: &str, _expires_secs: u64) -> Result<String> {
        Ok(format!("http://localhost:{}/files/{}", self.port, key))
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let path = format!("{}/{}", self.base_path, key);
        if std::path::Path::new(&path).exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }
}

pub fn create_storage_client(
    backend: &str,
    local_path: Option<&str>,
    port: u16,
    _r2_account_id: Option<&str>,
    _r2_access_key_id: Option<&str>,
    _r2_secret_access_key: Option<&str>,
    _r2_bucket_name: Option<&str>,
    _r2_public_url: Option<&str>,
) -> Result<Arc<dyn StorageClient>> {
    match backend {
        "local" => {
            let path = local_path.unwrap_or("./storage");
            Ok(Arc::new(LocalStorageClient::new(path, port)))
        }
        other => anyhow::bail!("Unsupported storage backend: {other}"),
    }
}
