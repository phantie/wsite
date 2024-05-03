#[derive(Debug)]
#[allow(unused)]
pub struct ServedFile<'a> {
    pub path: &'a str,
    pub size: &'a str,
}

pub fn file_response(file: &File) -> axum::response::Response {
    use axum::response::IntoResponse;
    let last_modified = httpdate::fmt_http_date(file.modified);
    let mime_type = mime_guess::from_path(&file.path.as_ref()).first_or_text_plain();
    // tracing::warn!("mime type {mime_type} derived from {:?}", &file.path);

    axum::http::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_str(mime_type.as_ref()).unwrap(),
        )
        .header(axum::http::header::LAST_MODIFIED, last_modified)
        .body(axum::body::boxed(axum::body::Full::<bytes::Bytes>::from(
            file.contents.clone(),
        )))
        .unwrap_or_else(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

#[derive(Debug)]
pub struct File {
    pub contents: Vec<u8>,
    pub request_path: String,
    pub path: Box<std::path::PathBuf>,
    pub modified: std::time::SystemTime,
}

#[derive(Debug, Clone, derived_deref::Deref)]
pub struct Cache {
    #[target]
    request_path_to_file: Arc<Mutex<clru::CLruCache<String, Arc<File>>>>,
    // no lru, all files cached to memory
    disk_path_to_file: Arc<RwLock<HashMap<Box<std::path::PathBuf>, Arc<File>>>>,
}

impl Cache {
    pub fn new(request_path_lru_size: std::num::NonZeroUsize) -> Self {
        Self {
            request_path_to_file: Arc::new(Mutex::new(clru::CLruCache::new(request_path_lru_size))),
            disk_path_to_file: Default::default(),
        }
    }
}

impl Cache {
    pub async fn get_request_path(&self, path: &str) -> Option<Arc<File>> {
        self.request_path_to_file
            .lock()
            .await
            .get(path)
            .map(Clone::clone)
    }

    pub async fn get_disk_path(&self, path: &std::path::PathBuf) -> Option<Arc<File>> {
        self.disk_path_to_file
            .read()
            .await
            .get(path)
            .map(Clone::clone)
    }

    pub async fn insert(&self, path: String, file: Arc<File>) {
        self.request_path_to_file
            .lock()
            .await
            .put(path, file.clone());
        self.disk_path_to_file
            .write()
            .await
            .insert(file.path.clone(), file);
    }
}

fn process_file(
    mut file: std::fs::File,
    file_path: std::path::PathBuf,
    request_path: String,
) -> File {
    use std::io::Read;
    let modified = file.metadata().unwrap().modified().unwrap();
    let mut contents = vec![];
    file.read_to_end(&mut contents).unwrap();
    File {
        contents,
        path: Box::new(file_path),
        request_path: request_path.clone(),
        modified,
    }
}

pub mod fallback {
    use crate::conf::Conf;
    use axum::{response::IntoResponse, Extension};

    use super::*;

    pub async fn fallback(
        uri: axum::http::Uri,
        Extension(cache): Extension<Cache>,
        Extension(conf): Extension<Conf>,
    ) -> axum::response::Response {
        let request_path = {
            let request_path = uri.to_string();
            request_path.trim_start_matches('/').trim().to_string()
        };

        if let Some(file) = cache.get_request_path(&request_path).await {
            tracing::info!("cache hit for request path: {request_path:?}");
            return file_response(&file);
        }

        let dir = std::path::Path::new(&conf.dir);
        let file_path = dir.join(request_path.clone());

        tracing::info!("Trying to serve: {:?}", file_path);

        let file_path = if file_path.is_file() {
            file_path
        } else {
            match &conf.fallback {
                Some(file_path) => {
                    let file_path = std::path::Path::new(file_path);

                    if file_path.is_file() {
                        file_path.to_path_buf()
                    } else {
                        return hyper::StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    }
                }
                None => return hyper::StatusCode::NOT_FOUND.into_response(),
            }
        };

        let file = cache.get_disk_path(&file_path).await;

        #[allow(unused)]
        let display_cache_keys = async {
            let lock = cache.lock().await;
            let keys = lock.iter().map(|(k, _)| k).collect::<Vec<_>>();
            tracing::warn!("cache keys: {keys:?}");
        };

        match file {
            None => {
                tracing::warn!("cache miss on file path: {file_path:?}");

                let file = std::fs::File::open(&file_path).expect("opens when exists");

                let file = process_file(file, file_path, request_path.clone());
                let response = file_response(&file);
                cache.insert(request_path, std::sync::Arc::new(file)).await;
                // display_cache_keys.await;
                response
            }
            Some(cached) => {
                tracing::warn!("cache hit on file path: {file_path:?}");
                // do not go to disk, reuse cached value
                cache.insert(request_path, cached.clone()).await;
                // display_cache_keys.await;
                file_response(&cached)
            }
        }
    }
}

pub static STATIC_DIR: &str = "static/";

// TODO join with fallback
pub mod serve_static {
    use super::{file_response, process_file, STATIC_DIR};
    use crate::routes::imports::*;

    #[axum_macros::debug_handler]
    pub async fn serve_static(Path(path): Path<String>) -> ApiResult<Response> {
        let file_path = std::path::PathBuf::from(STATIC_DIR).join(path.clone());
        let file = std::fs::File::open(&file_path);

        match file {
            Err(_e) => return Ok(IntoResponse::into_response(StatusCode::NOT_FOUND)),
            Ok(file) => Ok(file_response(&process_file(file, file_path, path.clone()))),
        }
    }
}
