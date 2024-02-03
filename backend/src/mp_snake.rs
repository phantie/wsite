use interfacing::snake::{LobbyName, UserName};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

pub type Con = std::net::SocketAddr;

#[derive(Default, Clone)]
pub struct PlayerUserNames(Arc<Mutex<bidirectional_map::Bimap<Con, UserName>>>);

impl PlayerUserNames {
    pub async fn try_insert(&self, un: UserName, con: Con) -> Result<(), ()> {
        // idempotent

        let mut lock = self.0.lock().await;

        if lock.contains_fwd(&con) {
            if lock.contains_rev(&un) && lock.get_fwd(&con).unwrap() != &un {
                Err(()) // occupied
            } else {
                lock.insert(con, un);
                Ok(())
            }
        } else {
            if lock.contains_rev(&un) {
                Err(()) // occupied
            } else {
                lock.insert(con, un);
                Ok(())
            }
        }
    }

    #[allow(unused)]
    pub async fn free(&self, un: UserName) {
        let mut lock = self.0.lock().await;
        if lock.contains_rev(&un) {
            lock.remove_rev(&un);
        }
    }

    pub async fn clean_con(&self, con: Con) {
        let mut lock = self.0.lock().await;
        if lock.contains_fwd(&con) {
            lock.remove_fwd(&con);
        }
    }
}

#[derive(Default, Clone)]
pub struct _PlayerUserNames(Arc<Mutex<(HashMap<Con, UserName>, HashMap<UserName, Con>)>>);

#[allow(dead_code, unused)]
impl _PlayerUserNames {
    pub async fn try_insert(&self, un: UserName, con: Con) -> Result<(), ()> {
        unimplemented!()
    }

    pub async fn free(&self, un: UserName) {
        let mut lock = self.0.lock().await;

        match lock.1.entry(un) {
            std::collections::hash_map::Entry::Occupied(entry) => {
                let con = entry.get().clone();
                entry.remove();
                lock.0.remove(&con);
            }
            std::collections::hash_map::Entry::Vacant(_) => {}
        }
    }

    pub async fn clean_con(&self, con: Con) {
        let mut lock = self.0.lock().await;
        lock.0.remove(&con).map(|un| lock.1.remove(&un));
    }
}

type ServerMsg = interfacing::snake::WsMsg<interfacing::snake::WsServerMsg>;
type Ch = tokio::sync::mpsc::UnboundedSender<ServerMsg>;

pub struct LobbyConState {
    pub ch: Ch,
    pub user_name: UserName,
    vote_start: bool,
}

impl LobbyConState {
    pub fn new(ch: Ch, un: UserName) -> Self {
        Self {
            ch,
            user_name: un,
            vote_start: false,
        }
    }
}

pub struct Lobby {
    #[allow(unused)]
    pub name: LobbyName,
    #[allow(unused)]
    pub players: HashMap<Con, LobbyConState>,
}

impl Lobby {
    pub fn new(name: LobbyName) -> Self {
        Self {
            name,
            players: Default::default(),
        }
    }

    fn join_player(&mut self, con: Con, ch: Ch, un: UserName) {
        self.players.insert(con, LobbyConState::new(ch, un));
    }

    #[allow(unused)]
    pub fn disjoin_player(&mut self, player: &Con) {
        self.players.remove(&player);
    }

    #[allow(unused)]
    fn prep(&self) -> interfacing::snake::lobby_change::LobbyPrep {
        use interfacing::snake::lobby_change::{LobbyPrep, Participant};

        LobbyPrep {
            participants: self
                .players
                .values()
                .map(
                    |LobbyConState {
                         user_name,
                         vote_start,
                         ..
                     }| Participant {
                        user_name: user_name.clone(),
                        vote_start: *vote_start,
                    },
                )
                .collect(),
        }
    }

    /// Broadcast message to all lobby participants
    #[allow(unused)]
    pub fn broadcast(&self, msg: ServerMsg) {
        self.players
            .values()
            .for_each(|LobbyConState { ch, .. }| ch.send(msg.clone()).unwrap_or(()));
    }
}

#[derive(derived_deref::Deref, Clone, Default)]
pub struct Lobbies(
    #[target] Arc<RwLock<HashMap<LobbyName, Arc<RwLock<Lobby>>>>>,
    Arc<RwLock<HashMap<Con, LobbyName>>>,
);

pub enum JoinLobbyError {
    // to the other
    AlreadyJoined(LobbyName),
    NotFound,
    // UserNameNotSet
}

// TODO maybe forbid deref
impl Lobbies {
    pub async fn joined_any(&self, con: Con) -> bool {
        self.1.read().await.get(&con).is_some()
    }

    // TODO verify
    #[allow(unused)]
    pub async fn remove_lobby(&self, lobby_name: LobbyName) {
        // while you hold this lock, noone else touches players
        let mut player_to_lobby = self.1.write().await;

        let _lock = self.read().await;
        let lobby = _lock.get(&lobby_name).expect("to be in sync");

        let players = &lobby.read().await.players;

        for (con, _) in players {
            player_to_lobby.remove(con);
        }

        self.write().await.remove(&lobby_name);
    }

    pub async fn disjoin_player(&self, player: Con) {
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
        player: Con,
        ch: Ch,
        un: UserName,
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
                        lobby.write().await.join_player(player, ch, un);
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
                    Err(JoinLobbyError::AlreadyJoined(lobby_name.clone()))
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
