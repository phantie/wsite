use interfacing::snake::{LobbyName, MsgId, UserName, WsMsg};
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
    ch: Ch,
    user_name: UserName,
}

impl LobbyConState {
    pub fn new(ch: Ch, un: UserName) -> Self {
        Self { ch, user_name: un }
    }
}

#[allow(unused)]
pub enum LobbyState {
    Prep { start_votes: HashMap<Con, bool> },
    Running,
}

pub struct Lobby {
    pub name: LobbyName,
    pub players: HashMap<Con, LobbyConState>,
    #[allow(unused)]
    pub state: LobbyState,
}

impl Lobby {
    pub fn new(name: LobbyName) -> Self {
        Self {
            name,
            players: Default::default(),
            state: LobbyState::Prep {
                start_votes: Default::default(),
            },
        }
    }

    fn join_player(&mut self, con: Con, ch: Ch, un: UserName) -> Result<(), String> {
        match &mut self.state {
            LobbyState::Prep { start_votes } => {
                self.players.insert(con, LobbyConState::new(ch, un));
                start_votes.insert(con, false);
                Ok(())
            }
            _ => Err("Illegal state".into()),
        }
    }

    #[allow(unused)]
    pub fn disjoin_player(&mut self, con: &Con) {
        self.players.remove(&con);
        match &mut self.state {
            LobbyState::Prep { start_votes } => {
                start_votes.remove(&con);
            }
            _ => {}
        }
    }

    #[allow(unused)]
    pub fn broadcast_state(&self) {
        self.broadcast(WsMsg::new(interfacing::snake::WsServerMsg::LobbyState(
            self.state(),
        )))
    }

    // include Id for the participant who's request triggered broadcast
    pub fn pinned_broadcast_state(&self, pin: MsgId, con: Con) {
        let send = WsMsg::new(interfacing::snake::WsServerMsg::LobbyState(self.state()));
        self.players
            .iter()
            .filter(|(_con, _)| con == **_con)
            .for_each(|(_, LobbyConState { ch, .. })| {
                ch.send(send.clone().id(pin.clone())).unwrap_or(())
            });
        self.players
            .iter()
            .filter(|(_con, _)| con != **_con)
            .for_each(|(_, LobbyConState { ch, .. })| ch.send(send.clone()).unwrap_or(()));
    }

    /// Broadcast message to all lobby participants
    fn broadcast(&self, msg: ServerMsg) {
        self.players
            .values()
            .for_each(|LobbyConState { ch, .. }| ch.send(msg.clone()).unwrap_or(()));
    }

    pub fn state(&self) -> interfacing::snake::lobby_state::LobbyState {
        use interfacing::snake::lobby_state::{LobbyPrep, LobbyPrepParticipant};

        match &self.state {
            LobbyState::Prep { start_votes } => {
                interfacing::snake::lobby_state::LobbyState::Prep(LobbyPrep {
                    participants: self
                        .players
                        .iter()
                        .map(
                            |(con, LobbyConState { user_name, .. })| LobbyPrepParticipant {
                                user_name: user_name.clone(),
                                vote_start: *start_votes.get(&con).expect("to be in sync"),
                            },
                        )
                        .collect(),
                })
            }
            LobbyState::Running => interfacing::snake::lobby_state::LobbyState::Running,
        }
    }

    fn all_voted_to_start(&self) -> Result<bool, String> {
        match &self.state {
            LobbyState::Prep { start_votes } => {
                Ok(start_votes.values().cloned().all(std::convert::identity))
            }
            _ => Err("Illegal state".into()),
        }
    }

    pub fn vote_start(&mut self, con: Con, value: bool) -> Result<(), String> {
        use std::collections::hash_map::Entry;

        match &mut self.state {
            LobbyState::Prep { start_votes } => match start_votes.entry(con) {
                Entry::Vacant(_) => Err("Player not found".into()),
                Entry::Occupied(mut entry) => {
                    entry.insert(value);

                    if self.all_voted_to_start().unwrap() {
                        self.state = LobbyState::Running;
                    }

                    Ok(())
                }
            },
            _ => Err("Illegal state".into()),
        }
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
    AlreadyStarted,
}

// TODO maybe forbid deref
impl Lobbies {
    pub async fn joined_lobby(&self, con: Con) -> Option<Arc<RwLock<Lobby>>> {
        match self.1.read().await.get(&con) {
            None => None,
            Some(ln) => Some(self.read().await[ln].clone()),
        }
    }

    pub async fn joined_any(&self, con: Con) -> bool {
        self.joined_lobby(con).await.is_some()
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

    pub async fn disjoin_player(&self, con: Con) {
        // while you hold this lock, noone else touches players
        let mut player_to_lobby = self.1.write().await;

        match player_to_lobby.get(&con) {
            None => {}
            Some(_lobby_name) => {
                let _lock = self.read().await;
                let lobby = _lock.get(_lobby_name).expect("to be in sync");
                player_to_lobby.remove(&con);
                lobby.write().await.disjoin_player(&con);
            }
        }
    }

    pub async fn join_player(
        &self,
        lobby_name: LobbyName,
        con: Con,
        ch: Ch,
        un: UserName,
    ) -> Result<(), JoinLobbyError> {
        // while you hold this lock, noone else touches players
        let mut player_to_lobby = self.1.write().await;

        match player_to_lobby.get(&con) {
            None => {
                let _lock = self.read().await;
                let lobby = _lock.get(&lobby_name);

                match lobby {
                    None => Err(JoinLobbyError::NotFound),
                    Some(lobby) => {
                        player_to_lobby.insert(con.clone(), lobby_name);
                        match lobby.write().await.join_player(con, ch, un) {
                            Ok(()) => Ok(()),
                            Err(_m) => Err(JoinLobbyError::AlreadyStarted),
                        }
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

    /// Get lobby by name
    pub async fn get(&self, name: &LobbyName) -> Option<Arc<RwLock<Lobby>>> {
        self.read().await.get(name).cloned()
    }

    /// Create lobby only if it's not already created
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
