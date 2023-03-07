#![allow(dead_code)]

type Id = String;
type Name = String;

type UserEntry = (Id, Name);

#[derive(Default, Clone)]
pub struct UserDatabase {
    pub data: Vec<UserEntry>,
}

impl UserDatabase {
    pub fn add_user(&mut self, entry: UserEntry) {
        self.data.push(entry);
    }

    pub fn get_user_by_id(&self, id: impl Into<Id>) -> Option<&UserEntry> {
        let id: Id = id.into();
        self.data.iter().find(|&(id_, _)| id_ == &id)
    }
}
