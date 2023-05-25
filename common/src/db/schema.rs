use bonsaidb::core::{
    document::{BorrowedDocument, Emit},
    schema::{
        view::ViewUpdatePolicy, Collection, MapReduce, ReduceResult, SerializedCollection, View,
        ViewMapResult, ViewMappedValue, ViewSchema,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Collection, Clone)]
#[collection(name = "shapes")]
pub struct Shape {
    pub sides: u32,
}

#[derive(Debug, Serialize, Deserialize, Collection, Clone)]
#[collection(name = "users",  views = [UserByUsername])]
pub struct User {
    pub username: String,
    pub password_hash: String,
}

#[derive(Debug, Clone, View)]
#[view(collection = User, key = String, value = u32, name = "by-username")]
pub struct UserByUsername;

impl ViewSchema for UserByUsername {
    type View = Self;
    type MappedKey<'doc> = <Self::View as View>::Key;

    fn update_policy(&self) -> ViewUpdatePolicy {
        ViewUpdatePolicy::Unique
    }
}

impl MapReduce for UserByUsername {
    fn map(&self, document: &BorrowedDocument<'_>) -> ViewMapResult<Self::View> {
        let user = User::document_contents(document)?;
        document.header.emit_key_and_value(user.username, 1)
    }

    fn reduce(
        &self,
        mappings: &[ViewMappedValue<Self::View>],
        _rereduce: bool,
    ) -> ReduceResult<Self::View> {
        Ok(mappings.iter().map(|mapping| mapping.value).sum())
    }
}

#[derive(Debug, Serialize, Deserialize, Collection, Clone)]
#[collection(name = "articles", views = [ArticleByPublicID])]
pub struct Article {
    pub title: String,
    pub public_id: String,
    pub markdown: String,
    pub draft: bool,
}

#[derive(Debug, Clone, View)]
#[view(collection = Article, key = String, value = u32, name = "by-public-id")]
pub struct ArticleByPublicID;

impl ViewSchema for ArticleByPublicID {
    type View = Self;
    type MappedKey<'doc> = <Self::View as View>::Key;

    fn update_policy(&self) -> ViewUpdatePolicy {
        ViewUpdatePolicy::Unique
    }
}

impl MapReduce for ArticleByPublicID {
    fn map(&self, document: &BorrowedDocument<'_>) -> ViewMapResult<Self::View> {
        let user = Article::document_contents(document)?;
        document.header.emit_key_and_value(user.public_id, 1)
    }

    fn reduce(
        &self,
        mappings: &[ViewMappedValue<Self::View>],
        _rereduce: bool,
    ) -> ReduceResult<Self::View> {
        Ok(mappings.iter().map(|mapping| mapping.value).sum())
    }
}

#[derive(Debug, Serialize, Deserialize, Collection, Clone)]
#[collection(name = "subscriptions", views = [SubscriptionByStatus, SubscriptionByToken])]
pub struct Subscription {
    pub name: String,
    pub email: domain::SubscriberEmail,
    pub status: String,
    pub token: String,
}

#[derive(Debug, Clone, View)]
#[view(collection = Subscription, key = String, value = u32, name = "by-status")]
pub struct SubscriptionByStatus;

impl ViewSchema for SubscriptionByStatus {
    type View = Self;
    type MappedKey<'doc> = <Self::View as View>::Key;
}

impl MapReduce for SubscriptionByStatus {
    fn map(&self, document: &BorrowedDocument<'_>) -> ViewMapResult<Self::View> {
        let subscription = Subscription::document_contents(document)?;
        document.header.emit_key_and_value(subscription.status, 1)
    }
}

#[derive(Debug, Clone, View)]
#[view(collection = Subscription, key = String, value = u32, name = "by-token")]
pub struct SubscriptionByToken;

impl ViewSchema for SubscriptionByToken {
    type View = Self;
    type MappedKey<'doc> = <Self::View as View>::Key;

    fn update_policy(&self) -> ViewUpdatePolicy {
        ViewUpdatePolicy::Unique
    }
}

impl MapReduce for SubscriptionByToken {
    fn map(&self, document: &BorrowedDocument<'_>) -> ViewMapResult<Self::View> {
        let subscription = Subscription::document_contents(document)?;
        document.header.emit_key_and_value(subscription.token, 1)
    }
}
