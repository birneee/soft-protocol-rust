use std::path::PathBuf;
use soft_shared_lib::error;
use tokio::fs::File;
use soft_shared_lib::error::ErrorType;

pub struct FileSandbox {
    served_dir: PathBuf
}

/// allow only access to files from the specified served directory
impl FileSandbox {

    pub fn new(served_dir: PathBuf) -> Self {
        FileSandbox { served_dir }
    }

    /// only server files from the public directory
    pub async fn get_file(&self, file_name: String) -> error::Result<File> {
        if file_name.starts_with("/") {
            return Err(ErrorType::FileNotFound);
        }
        if file_name.contains("..") {
            return Err(ErrorType::FileNotFound);
        }
        let path = self.served_dir.join(file_name);
        if !path.starts_with(&self.served_dir){
            return Err(ErrorType::FileNotFound);
        }
        if path.is_dir() {
            return Err(ErrorType::FileNotFound);
        }
        return Ok(File::open(path).await?);
    }
}