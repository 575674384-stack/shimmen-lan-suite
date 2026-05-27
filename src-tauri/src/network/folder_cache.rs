use crate::models::SharedFolder;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type RemoteFolderCache = Arc<Mutex<HashMap<String, Vec<SharedFolder>>>>;

pub fn create_folder_cache() -> RemoteFolderCache {
    Arc::new(Mutex::new(HashMap::new()))
}
