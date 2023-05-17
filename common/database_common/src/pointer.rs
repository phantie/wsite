#![allow(dead_code)]

use std::marker::PhantomData;

use bonsaidb::core::schema::Collection;

pub struct DatabasePointer<C>
where
    C: Collection,
{
    name: String,
    _c: PhantomData<C>,
}

impl<C> DatabasePointer<C>
where
    C: Collection,
{
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            _c: PhantomData,
        }
    }
}
