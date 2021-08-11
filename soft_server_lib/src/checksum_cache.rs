use soft_shared_lib::field_types::Checksum;
use tokio::io::BufReader;
use tokio::fs::File;
use soft_shared_async_lib::helper::sha256_helper::generate_checksum;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::os::linux::fs::MetadataExt;
use ttl_cache::TtlCache;
use std::time::Duration;

const MAX_ENTRIES: usize = 100;
/// TODO increase for production use
const ENTRY_TTL: Duration = Duration::from_secs(120);

type MTime = i64;

enum CacheEntry {
    Generating(MTime),
    Ready(Checksum, MTime),
}

pub struct ChecksumCache {
    cache: Arc<Mutex<TtlCache<String, CacheEntry>>>,
}

impl ChecksumCache {
    pub fn new() -> Arc<ChecksumCache> {
        Arc::new(ChecksumCache {
            cache: Arc::new(Mutex::new(TtlCache::new(MAX_ENTRIES))),
        })
    }

    /// Generate the checksum for that file or read if from cache
    /// None if still processing
    /// TODO make function return a shared Future instead of an Option type
    pub async fn get_checksum(self: Arc<Self>, file_name: &str, file: File) -> Option<Checksum> {
        let mut reader = BufReader::new(file);
        let current_mtime = reader.get_ref().metadata().await.unwrap().st_mtime();
        let mut cache = self.cache.lock().await;
        match cache.get(file_name) {
            Some(CacheEntry::Generating(cache_mtime)) => {
                if *cache_mtime == current_mtime {
                    return None;
                } else {
                    log::debug!("file {} changed", file_name);
                    cache.remove(file_name);
                }
            }
            Some(CacheEntry::Ready(checksum, cache_mtime)) => {
                if *cache_mtime == current_mtime {
                    return Some(checksum.clone());
                } else {
                    log::debug!("file {} changed", file_name);
                    cache.remove(file_name);
                }
            }
            _ => {}
        }

        // start generating checksum in own task
        let entry = CacheEntry::Generating(current_mtime);
        cache.insert(String::from(file_name), entry, ENTRY_TTL);
        drop(cache); // unlock
        let checksum_cache = self.clone();
        let file_name= String::from(file_name);
        tokio::spawn(async move {
            log::debug!("generating checksum for {}", file_name);
            let checksum = generate_checksum(&mut reader).await;
            let mut cache = checksum_cache.cache.lock().await;
            if let Some(&mut CacheEntry::Generating(mtime)) = cache.get_mut(&file_name) {
                if current_mtime == mtime {
                    cache.insert(file_name.clone(), CacheEntry::Ready(checksum, current_mtime), ENTRY_TTL);
                    log::debug!("checksum for {} is ready", file_name);
                }
            }
        });

        return None;
    }
}
