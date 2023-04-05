#![allow(dead_code)]
pub trait Url {
    fn prefix(&self) -> &str {
        ""
    }
    fn postfix(&self) -> &str;
}

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

    pub fn complete_with_base(&self, base_url: &str) -> String {
        format!("{}{}", base_url, self.complete())
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    pub fn postfix(&self) -> &str {
        &self.postfix
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
