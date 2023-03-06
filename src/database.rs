#![allow(dead_code)]

type Id = String;
type Name = String;

type UserEntry = (Id, Name);

#[derive(Default, Clone)]
pub struct UserDatabase {
    pub data: Vec<UserEntry>,
}

impl UserDatabase {
    fn add_user(&mut self, entry: UserEntry) {
        self.data.push(entry);
    }

    fn get_user_by_id(&self, id: Id) -> Option<&UserEntry> {
        self.data.iter().find(|&(id_, _)| id_ == &id)
    }
}
