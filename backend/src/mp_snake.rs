use interfacing::snake::LobbyName;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Player {
    pub id: std::net::SocketAddr,
}

#[derive(Clone)]
pub struct Lobby {
    #[allow(unused)]
    pub name: LobbyName,
    #[allow(unused)]
    pub players: HashSet<Player>,
}

impl Lobby {
    pub fn new(name: LobbyName) -> Self {
        Self {
            name,
            players: Default::default(),
        }
    }

    fn join_player(&mut self, player: Player) {
        self.players.insert(player);
    }

    #[allow(unused)]
    pub fn disjoin_player(&mut self, player: &Player) {
        self.players.remove(&player);
    }
}

#[derive(Clone, Default, derived_deref::Deref)]
pub struct Lobbies(
    #[target] Arc<RwLock<HashMap<LobbyName, Arc<RwLock<Lobby>>>>>,
    Arc<RwLock<HashMap<Player, LobbyName>>>,
);

pub enum JoinLobbyError {
    // to the other
    AlreadyJoined(LobbyName),
    NotFound,
    // UserNameNotSet
}

// TODO forbid deref
// TODO expose remove lobby
impl Lobbies {
    pub async fn disjoin_player(&self, player: Player) {
        // while you hold this lock, noone else touches players
        let mut player_to_lobby = self.1.write().await;

        match player_to_lobby.get(&player) {
            None => {}
            Some(_lobby_name) => {
                let _lock = self.read().await;
                let lobby = _lock.get(_lobby_name).expect("to be in sync");
                player_to_lobby.remove(&player);
                lobby.write().await.disjoin_player(&player);
            }
        }
    }

    pub async fn join_player(
        &self,
        lobby_name: LobbyName,
        player: Player,
    ) -> Result<(), JoinLobbyError> {
        // while you hold this lock, noone else touches players
        let mut player_to_lobby = self.1.write().await;

        match player_to_lobby.get(&player) {
            None => {
                let _lock = self.read().await;
                let lobby = _lock.get(&lobby_name);

                match lobby {
                    None => Err(JoinLobbyError::NotFound),
                    Some(lobby) => {
                        player_to_lobby.insert(player.clone(), lobby_name);
                        lobby.write().await.join_player(player);
                        Ok(())
                    }
                }
            }
            Some(_lobby_name) => {
                // idempotency
                if lobby_name == *_lobby_name {
                    // don't need to check lobby, since it must be in sync
                    Ok(())
                } else {
                    Err(JoinLobbyError::AlreadyJoined(_lobby_name.clone()))
                }
            }
        }
    }

    pub async fn get(&self, name: &LobbyName) -> Option<Arc<RwLock<Lobby>>> {
        self.read().await.get(name).cloned()
    }

    pub async fn insert_if_missing(&self, lobby: Lobby) -> Result<(), String> {
        use std::collections::hash_map::Entry;
        let mut w_lock = self.write().await;

        match w_lock.entry(lobby.name.clone()) {
            Entry::Occupied(_) => Err("Lobby with this name already exists".into()),
            Entry::Vacant(_) => {
                w_lock.insert(lobby.name.clone(), Arc::new(RwLock::new(lobby)));
                Ok(())
            }
        }
    }

    #[allow(dead_code)]
    async fn insert(&self, lobby: Lobby) {
        self.write()
            .await
            .insert(lobby.name.clone(), Arc::new(RwLock::new(lobby)));
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
