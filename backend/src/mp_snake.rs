use interfacing::snake::{LobbyName, UserName};

#[derive(Clone)]
pub struct Player {
    #[allow(unused)]
    name: UserName,
}

#[derive(Clone)]
pub struct Lobby {
    #[allow(unused)]
    pub name: LobbyName,
    #[allow(unused)]
    players: Vec<Player>,
}

impl Lobby {
    pub fn new(name: LobbyName) -> Self {
        Self {
            name,
            players: vec![],
        }
    }
}

// TODO Value In HashMap (Lobby) Should be wrapped in RwLock or Something
// to increase concurrency by accessing through outer RwLock using read
type Inner = std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<LobbyName, Lobby>>>;

#[derive(Clone, Default)]
pub struct Lobbies(Inner);

impl std::ops::Deref for Lobbies {
    type Target = Inner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Lobbies {
    pub async fn get(&self, name: &LobbyName) -> Option<Lobby> {
        self.read().await.get(name).cloned()
    }

    pub async fn insert_if_missing(&self, lobby: Lobby) -> Result<(), String> {
        use std::collections::hash_map::Entry;
        let mut w_lock = self.write().await;

        match w_lock.entry(lobby.name.clone()) {
            Entry::Occupied(_) => Err("Lobby with this name already exists".into()),
            Entry::Vacant(_) => {
                w_lock.insert(lobby.name.clone(), lobby);
                Ok(())
            }
        }
    }

    #[allow(dead_code)]
    async fn insert(&self, lobby: Lobby) {
        self.write().await.insert(lobby.name.clone(), lobby);
    }
}

pub mod ws {
    use interfacing::snake::UserName;

    #[allow(unused)]
    pub type Cons<S> =
        std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<std::net::SocketAddr, S>>>;

    #[derive(Clone, Default)]
    pub struct State {
        #[allow(unused)]
        pub user_name: Option<UserName>,
    }
}
