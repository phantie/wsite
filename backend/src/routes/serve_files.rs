use crate::routes::imports::*;

#[axum_macros::debug_handler]
pub async fn serve_static(Path(path): Path<String>) -> ApiResult<Response> {
    let static_dir = "static/";
    let path = std::path::PathBuf::from(static_dir).join(path);

    match std::fs::read(&path) {
        Err(_e) => return Ok(IntoResponse::into_response(StatusCode::NOT_FOUND)),
        Ok(file) => {
            let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
            return Ok(file_response(file, &path, modified));
        }
    }
}

pub async fn fallback(uri: axum::http::Uri) -> Response {
    let path = uri.to_string();
    let path = path.trim_start_matches('/');

    match FRONTEND_DIR.get_file(path) {
        None => IntoResponse::into_response(axum::response::Html(INDEX_HTML)),
        Some(file) => {
            let modified = file.metadata().unwrap().modified();
            file_response(file.contents(), path, modified)
        }
    }
}

pub fn file_response(
    contents: impl Into<axum::body::Full<bytes::Bytes>>,
    path: impl AsRef<std::path::Path>,
    modified: std::time::SystemTime,
) -> axum::response::Response {
    let last_modified = httpdate::fmt_http_date(modified);
    let mime_type = mime_guess::from_path(path).first_or_text_plain();
    axum::http::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderValue::from_str(mime_type.as_ref()).unwrap(),
        )
        .header(axum::http::header::LAST_MODIFIED, last_modified)
        .body(axum::body::boxed(contents.into()))
        .unwrap_or_else(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

static FRONTEND_DIR: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/../frontend/dist/");

static INDEX_HTML: &str = include_str!("../../../frontend/dist/index.html");
