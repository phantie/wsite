use crate::routes::imports::*;
use hyper::{Body, Request};
use tower::ServiceExt;
// use std::{
// use axum::{headers::Range, TypedHeader};
//     io::{Read, Seek, SeekFrom},
//     ops::Bound,
//     str::FromStr,
// };

// #[allow(unused)]
// pub async fn video(TypedHeader(range): TypedHeader<Range>) -> StatusCode {
//     let video_path = "flower.webm";

//     let chunk_size = 10u64.pow(6);

//     let (start, end) = range.iter().next().unwrap();

//     let response = match (start, end) {
//         (Bound::Included(start), Bound::Unbounded) => {
//             let mut f = std::fs::File::open(video_path).expect("open file");
//             let file_size = f.metadata().unwrap().len();
//             let end = u64::min(start + chunk_size, file_size - 1);

//             f.seek(SeekFrom::Start(start)).unwrap();
//             let mut buf = vec![0; chunk_size as usize];
//             f.read_exact(&mut buf).unwrap();
//             let content_length = end - start + 1;

//             let return_range = axum::headers::Range::bytes(start..end).unwrap();
//             let accept_range = axum::headers::AcceptRanges::bytes();
//             let content_length = axum::headers::ContentLength(content_length);
//             let content_type = axum::headers::ContentType::from_str("video/webm");

//             (return_range, accept_range, content_length, content_type, )
//         }
//         _ => unimplemented!(),
//     };

//     StatusCode::NOT_FOUND
// }

#[allow(unused)]
pub async fn video(request: Request<Body>) -> impl IntoResponse {
    let video_path = "flower.webm";
    let serve_file = tower_http::services::fs::ServeFile::new(video_path);
    serve_file.oneshot(request).await
}
