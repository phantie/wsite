pub trait Url {
    fn prefix(&self) -> &str {
        ""
    }
    fn postfix(&self) -> &str;
}

#[derive(Clone)]
pub struct RelativePath {
    prefix: String,
    postfix: String,
    complete: String,
}

impl RelativePath {
    pub fn new(prefix: String, postfix: String) -> Self {
        Self {
            complete: format!("{}{}", prefix, postfix),
            prefix,
            postfix,
        }
    }

    pub fn complete(&self) -> &str {
        &self.complete
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn postfix(&self) -> &str {
        &self.postfix
    }

    pub fn with_base(&self, base_url: &str) -> FullPath {
        FullPath::new(base_url.to_owned(), self.clone())
    }
}

pub struct FullPath {
    base_url: String,
    relative_path: RelativePath,
    complete: String,
}

impl FullPath {
    fn new(base_url: String, relative_path: RelativePath) -> Self {
        Self {
            complete: format!("{}{}", &base_url, relative_path.complete()),
            base_url,
            relative_path,
        }
    }

    pub fn complete(&self) -> &str {
        &self.complete
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn relative_path(&self) -> &RelativePath {
        &self.relative_path
    }
}

pub trait Get: Url {
    fn get(&self) -> RelativePath {
        RelativePath::new(self.prefix().to_owned(), self.postfix().to_owned())
    }
}

pub trait Post: Url {
    fn post(&self) -> RelativePath {
        RelativePath::new(self.prefix().to_owned(), self.postfix().to_owned())
    }
}
