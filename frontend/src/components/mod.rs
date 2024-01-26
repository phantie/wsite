#![allow(non_upper_case_globals)]

// TODO switch.rs uses it, refactor
pub mod imports;

mod articles;
mod colored;
mod default_styling;
mod error;
mod header;
mod login;
mod markdown;
mod markdown_preview;
mod markdown_preview_page;
mod online;
mod online_ctx;
mod post;
pub mod snake;
mod title;

pub mod admin;
pub use articles::*;
pub use colored::Colored;
pub use default_styling::DefaultStyling;
pub use error::Error;
pub use header::Header;
pub use login::Login;
pub use markdown::Markdown;
pub use markdown_preview::MarkdownPreview;
pub use markdown_preview_page::MarkdownPreviewPage;
pub use online::Online;
pub use online_ctx::{OnlineCtx, OnlineCtxSub, WithOnline};
pub use post::Post;
pub use snake::comp::Snake;
pub use title::PageTitle;

mod state;
pub mod theme;
