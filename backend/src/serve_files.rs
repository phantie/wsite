pub static FRONTEND_DIR: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/../frontend/dist/");

pub static INDEX_HTML: &str = include_str!("../../frontend/dist/index.html");

pub static STATIC_DIR: &str = "static/";

#[derive(Debug)]
#[allow(unused)]
pub struct ServedFile<'a> {
    pub path: &'a str,
    pub size: &'a str,
}
