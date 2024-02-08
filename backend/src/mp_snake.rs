use interfacing::snake::{LobbyName, MsgId, UserName, WsMsg};
use interfacing::snake_domain as domain;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

// could be any, granting uniqueness to ws connection among all
pub type Con = u16;

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

// lobby parameters
#[derive(Default)]
pub struct PrepLobbyState {
    // should contain all players in lobby
    start_votes: HashMap<Con, bool>,
}

impl PrepLobbyState {
    fn to_running(&self) -> RunningLobbyState {
        self.into()
    }

    fn join_con(&mut self, con: Con) {
        self.start_votes.insert(con, false);
    }

    fn remove_con(&mut self, con: &Con) {
        self.start_votes.remove(con);
    }

    fn vote_start(&mut self, con: Con, vote: bool) {
        // expected to already contain the con
        if self.start_votes.contains_key(&con) {
            self.start_votes.insert(con, vote);
        }
    }

    fn all_voted_to_start(&self) -> bool {
        self.start_votes
            .values()
            .cloned()
            .all(std::convert::identity)
    }
}

pub struct RunningLobbyState {
    // TODO merge into "con to con state"
    snakes: HashMap<Con, domain::Snake>,
    foods: domain::Foods,
    counter: u32,
    cons: std::collections::HashSet<Con>,
}

impl From<&PrepLobbyState> for RunningLobbyState {
    fn from(PrepLobbyState { start_votes }: &PrepLobbyState) -> Self {
        #[allow(unused)]
        use domain::{Direction, Food, Foods, Pos, Sections, Snake};

        let cons = start_votes.keys().cloned().collect::<HashSet<_>>();

        let snakes = {
            let mut snakes = vec![];

            for (i, con) in cons.iter().cloned().enumerate() {
                let sections = Sections::from_directions(
                    Pos::new(i as _, 0),
                    (0..3).into_iter().map(|_| Direction::Up),
                );

                let snake = Snake {
                    sections,
                    direction: Direction::Up,
                };

                snakes.push((con, snake));
            }

            snakes.into_iter().collect()
        };

        let foods = Foods { values: vec![] };

        Self {
            snakes,
            foods,
            counter: 0,
            cons,
        }
    }
}

impl RunningLobbyState {
    fn advance(&mut self) {
        use domain::AdvanceResult;

        self.counter += 1;

        // indeces to remove
        let mut rm = vec![];
        let mut add_foods = vec![];

        let other_snakes = self.snakes.clone();
        for (i, snake) in self.snakes.values_mut().enumerate() {
            let other_snakes = other_snakes
                .values()
                .enumerate()
                .filter(|(_i, _)| *_i != i)
                .map(|(_, snake)| snake.clone())
                .collect::<Vec<_>>();

            match snake.advance(&mut self.foods, other_snakes.as_slice()) {
                AdvanceResult::Success => {}
                AdvanceResult::BitYaSelf | AdvanceResult::BitSomeone => {
                    rm.push(i);

                    add_foods.extend(
                        snake
                            .iter_vertices()
                            .map(|domain::Pos { x, y }| domain::Food::new(x, y)),
                    );
                }
            }
        }

        let mut idx = 0;
        self.snakes.retain(|_, _| {
            let retain = !rm.contains(&idx);
            idx += 1;
            retain
        });

        self.foods.extend(add_foods.into_iter());
    }

    fn set_con_direction(&mut self, con: Con, direction: domain::Direction) {
        if self.snakes.contains_key(&con) {
            self.snakes
                .get_mut(&con)
                .unwrap()
                .set_direction(direction)
                .unwrap_or(());
            tracing::info!("set direction {:?}", direction);
        }
    }

    // no join_con because joining midgame is forbidden

    fn remove_con(&mut self, con: &Con) {
        self.cons.remove(con);
        self.snakes.remove(con);
    }
}

pub enum LobbyState {
    Prep(PrepLobbyState),
    Running(RunningLobbyState),
    // terminated is scheduled for clean up
    Terminated,
}

pub struct Lobby {
    pub name: LobbyName,
    pub players: HashMap<Con, LobbyConState>,
    pub state: LobbyState,

    ch: Option<tokio::sync::mpsc::UnboundedSender<LobbyCtrlMsg>>,
    // TODO maybe ship with RunningLobbyState
    _loop_handle: Option<tokio::task::AbortHandle>,
}

#[derive(Debug)]
pub enum LobbyMsg {
    Advance,
}

pub enum LobbiesMsg {
    RemoveLobby(LobbyName),
}

pub enum LobbyCtrlMsg {
    LobbyMsg(LobbyMsg),
    LobbiesMsg(LobbiesMsg),
}

impl Lobby {
    pub fn new(name: LobbyName) -> Self {
        Self {
            name,
            players: Default::default(),
            state: LobbyState::Prep(PrepLobbyState::default()),

            ch: None,
            _loop_handle: None,
        }
    }

    pub fn state(&self, receiver: Con) -> interfacing::snake::LobbyState {
        use interfacing::snake::lobby_state::{LobbyPrep, LobbyPrepParticipant};

        match &self.state {
            // TODO it cannot impl From because State itself participates in calculation
            // one way would be to duplicate user_names to PrepLobbyState
            LobbyState::Prep(PrepLobbyState { start_votes }) => {
                interfacing::snake::LobbyState::Prep(LobbyPrep {
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

            LobbyState::Running(RunningLobbyState {
                counter,
                cons,
                snakes,
                foods,
                ..
            }) => {
                use interfacing::snake::lobby_state::LobbyRunning;

                let con: Con = receiver;

                let snake = snakes
                    .into_iter()
                    .find(|(_con, _)| **_con == con)
                    .map(|(_, snake)| snake.clone())
                    // /*e TODO fix default snake because null here is unsupported */
                    .unwrap_or_else(|| domain::Snake {
                        sections: domain::Sections::from_directions(
                            domain::Pos { x: 0, y: 0 },
                            [domain::Direction::Up],
                        ),
                        direction: domain::Direction::Up,
                    });

                let other_snakes = snakes
                    .into_iter()
                    .filter(|(_con, _)| **_con != con)
                    .map(|(_, snake)| snake.clone())
                    .collect::<Vec<_>>();

                interfacing::snake::LobbyState::Running(LobbyRunning {
                    counter: *counter,
                    player_counter: cons.len() as _,
                    domain: domain::Domain {
                        snake,
                        foods: foods.clone(),
                        other_snakes,
                        // TODO support boundaries
                        boundaries: domain::Pos::new(0, 0).boundaries_in_radius(10, 10),
                    },
                })
            }
            LobbyState::Terminated => interfacing::snake::LobbyState::Terminated,
        }
    }

    pub fn begin(&mut self) -> Result<(), String> {
        match &mut self.state {
            LobbyState::Prep(s) => {
                self.state = LobbyState::Running(s.to_running());

                let ch = self.ch.clone().expect("set up channel");
                self._loop_handle.replace(
                    tokio::spawn(async move {
                        // TODO should be swaped, or added larger pause before loop
                        loop {
                            ch.send(LobbyCtrlMsg::LobbyMsg(LobbyMsg::Advance)).unwrap();
                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        }
                    })
                    .abort_handle(),
                );

                Ok(())
            }
            _ => Err("Illegal state".into()),
        }
    }

    pub fn stop(&mut self) {
        match &self.state {
            LobbyState::Running { .. } => {
                self._loop_handle.take().expect("set up channel").abort();
                self.ch.take();

                self.state = LobbyState::Terminated;
            }
            _ => {}
        }
    }

    pub fn vote_start(&mut self, con: Con, value: bool) -> Result<(), String> {
        match &mut self.state {
            LobbyState::Prep(s) => {
                s.vote_start(con, value);
                if s.all_voted_to_start() {
                    self.begin().unwrap();
                }

                Ok(())
            }
            _ => Err("Illegal state".into()),
        }
    }

    pub fn set_con_direction(
        &mut self,
        con: Con,
        direction: domain::Direction,
    ) -> Result<(), String> {
        match &mut self.state {
            LobbyState::Running(s) => {
                s.set_con_direction(con, direction);
                Ok(())
            }
            _ => Err("Illegal state".into()),
        }
    }

    fn join_con(&mut self, con: Con, ch: Ch, un: UserName) -> Result<(), String> {
        match &mut self.state {
            LobbyState::Prep(s) => {
                self.players.insert(con, LobbyConState::new(ch, un));
                s.join_con(con);
                Ok(())
            }
            _ => Err("Illegal state".into()),
        }
    }

    pub fn disjoin_con(&mut self, con: &Con) {
        self.players.remove(&con);
        match &mut self.state {
            LobbyState::Prep(s) => {
                s.remove_con(con);
            }

            LobbyState::Running(s) => {
                // when everyone quits from running game, remove lobby
                if self.players.is_empty() {
                    let send = LobbiesMsg::RemoveLobby(self.name.clone());
                    self.ch
                        .as_ref()
                        .unwrap()
                        .send(LobbyCtrlMsg::LobbiesMsg(send))
                        .unwrap();
                }
                s.remove_con(con);
            }

            LobbyState::Terminated => {}
        }
    }
}

// message passing impl
impl Lobby {
    #[must_use = "to use message passing"]
    pub fn set_ch(mut self, ch: tokio::sync::mpsc::UnboundedSender<LobbyCtrlMsg>) -> Self {
        self.ch.replace(ch);
        self
    }

    pub fn handle_message(&mut self, msg: LobbyMsg) {
        match &mut self.state {
            LobbyState::Prep { .. } => {
                tracing::warn!("unhandled message {msg:?}")
            }
            LobbyState::Running(s) => match msg {
                LobbyMsg::Advance => {
                    s.advance();
                    self.broadcast_state();
                }
            },
            LobbyState::Terminated => {
                tracing::warn!("unhandled message {msg:?}")
            }
        }
    }
}

// broadcast impl
impl Lobby {
    pub fn broadcast_state(&self) {
        let send = |con| WsMsg::new(interfacing::snake::WsServerMsg::LobbyState(self.state(con)));

        self.players
            .iter()
            .for_each(|(_con, LobbyConState { ch, .. })| ch.send(send(*_con)).unwrap_or(()));
    }

    // include Id for the participant who's request triggered broadcast
    pub fn pinned_broadcast_state(&self, pin: MsgId, con: Con) {
        let send = |con| WsMsg::new(interfacing::snake::WsServerMsg::LobbyState(self.state(con)));

        self.players
            .iter()
            .filter(|(_con, _)| con == **_con)
            .for_each(|(_con, LobbyConState { ch, .. })| {
                ch.send(send(*_con).id(pin.clone())).unwrap_or(())
            });

        self.players
            .iter()
            .filter(|(_con, _)| con != **_con)
            .for_each(|(_con, LobbyConState { ch, .. })| ch.send(send(*_con)).unwrap_or(()));
    }

    pub fn broadcast_state_except(&self, con: Con) {
        let send = |con| WsMsg::new(interfacing::snake::WsServerMsg::LobbyState(self.state(con)));

        self.players
            .iter()
            .filter(|(_con, _)| con != **_con)
            .for_each(|(_con, LobbyConState { ch, .. })| ch.send(send(*_con)).unwrap_or(()));
    }

    /// Broadcast message to all lobby participants
    #[allow(unused)]
    fn broadcast(&self, msg: ServerMsg) {
        self.players
            .values()
            .for_each(|LobbyConState { ch, .. }| ch.send(msg.clone()).unwrap_or(()));
    }
}

type LobbyMessagePasserAbortHandle = tokio::task::AbortHandle;

#[derive(Clone, Default)]
pub struct Lobbies(
    Arc<RwLock<HashMap<LobbyName, Arc<RwLock<Lobby>>>>>,
    Arc<RwLock<HashMap<Con, LobbyName>>>,
    Arc<RwLock<HashMap<LobbyName, LobbyMessagePasserAbortHandle>>>,
);

pub enum JoinLobbyError {
    // to the other
    AlreadyJoined(LobbyName),
    NotFound,
    AlreadyStarted,
}

impl Lobbies {
    pub async fn lobby_names(&self) -> Vec<LobbyName> {
        self.0.read().await.keys().cloned().into_iter().collect()
    }

    #[allow(dead_code)]
    pub async fn lobby_state(&self, con: Con) -> Option<interfacing::snake::LobbyState> {
        match self.joined_lobby(con).await {
            None => None, // player not in any lobby
            Some(lobby) => Some(lobby.read().await.state(con)),
        }
    }

    pub async fn joined_lobby(&self, con: Con) -> Option<Arc<RwLock<Lobby>>> {
        match self.1.read().await.get(&con) {
            None => None, // player not in any lobby
            Some(ln) => Some(self.0.read().await[ln].clone()),
        }
    }

    pub async fn joined_any(&self, con: Con) -> bool {
        self.joined_lobby(con).await.is_some()
    }

    // Remove lobby if exists
    pub async fn remove_lobby(&self, lobby_name: LobbyName) {
        // while you hold this lock, noone else touches players
        let mut con_to_lobby = self.1.write().await;

        {
            let _lock = self.0.read().await;

            let lobby = match _lock.get(&lobby_name) {
                None => return,
                Some(lobby) => lobby,
            };

            let mut _lobby_lock = lobby.write().await;

            _lobby_lock.stop();

            let players = &_lobby_lock.players;

            for (con, _) in players {
                con_to_lobby.remove(con);
            }
        }

        self.0.write().await.remove(&lobby_name);
        self.2
            .write()
            .await
            .remove(&lobby_name)
            .expect("to be in sync")
            .abort();
    }

    pub async fn disjoin_con(&self, con: Con) {
        // while you hold this lock, noone else touches players
        let mut con_to_lobby = self.1.write().await;

        match con_to_lobby.get(&con) {
            None => {}
            Some(_lobby_name) => {
                let _lock = self.0.read().await;
                let lobby = _lock.get(_lobby_name).expect("to be in sync");
                con_to_lobby.remove(&con);
                lobby.write().await.disjoin_con(&con);
                lobby.read().await.broadcast_state();
            }
        }
    }

    /// Try join con to specified lobby
    /// Con associates with
    ///     - Ch (WsServerMessage channel)
    ///     - UserName (Cannot be changed while in lobby)
    /// On success return lobby state, as an informative Ack
    pub async fn join_con(
        &self,
        lobby_name: LobbyName,
        con: Con,
        ch: Ch,
        un: UserName,
    ) -> Result<interfacing::snake::LobbyState, JoinLobbyError> {
        // while you hold this lock, noone else touches players
        let mut con_to_lobby = self.1.write().await;

        match con_to_lobby.get(&con) {
            None => {
                let _lock = self.0.read().await;
                let lobby = _lock.get(&lobby_name);

                match lobby {
                    None => Err(JoinLobbyError::NotFound),
                    Some(lobby) => {
                        con_to_lobby.insert(con.clone(), lobby_name);
                        let mut lock = lobby.write().await;
                        match lock.join_con(con, ch, un) {
                            Ok(()) => {
                                lock.broadcast_state_except(con);
                                Ok(lock.state(con))
                            }
                            Err(_m) => Err(JoinLobbyError::AlreadyStarted),
                        }
                    }
                }
            }
            Some(_lobby_name) => {
                // idempotency
                if lobby_name == *_lobby_name {
                    // don't need to check lobby, since it must be in sync

                    Ok(self
                        .get(_lobby_name)
                        .await
                        .unwrap() // TODO verify no in between changes
                        .read()
                        .await
                        .state(con))
                } else {
                    Err(JoinLobbyError::AlreadyJoined(lobby_name.clone()))
                }
            }
        }
    }

    /// Get lobby by name
    pub async fn get(&self, name: &LobbyName) -> Option<Arc<RwLock<Lobby>>> {
        self.0.read().await.get(name).cloned()
    }

    /// Create lobby only if it's not already created
    pub async fn insert_if_missing(&self, lobby: Lobby) -> Result<(), String> {
        use std::collections::hash_map::Entry;
        let mut w_lock = self.0.write().await;

        match w_lock.entry(lobby.name.clone()) {
            Entry::Occupied(_) => Err("Lobby with this name already exists".into()),
            Entry::Vacant(_) => {
                let lobby_name = lobby.name.clone();

                let (s, mut r) = tokio::sync::mpsc::unbounded_channel::<LobbyCtrlMsg>();
                let lobby = Arc::new(RwLock::new(lobby.set_ch(s)));

                {
                    w_lock.insert(lobby_name.clone(), lobby.clone());
                }

                {
                    let lobbies = self.clone();
                    let lobby_msg_passer_handle = tokio::spawn(async move {
                        while let Some(msg) = r.recv().await {
                            match msg {
                                LobbyCtrlMsg::LobbyMsg(msg) => {
                                    lobby.write().await.handle_message(msg)
                                }
                                LobbyCtrlMsg::LobbiesMsg(msg) => match msg {
                                    LobbiesMsg::RemoveLobby(ln) => {
                                        lobbies.remove_lobby(ln).await;
                                    }
                                },
                            }
                        }
                    })
                    .abort_handle();
                    self.2
                        .write()
                        .await
                        .insert(lobby_name, lobby_msg_passer_handle);
                }

                Ok(())
            }
        }
    }
}

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

pub mod ws {
    use interfacing::snake::UserName;

    #[allow(unused)]
    pub type Cons<S> =
        std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<std::net::SocketAddr, S>>>;

    #[derive(Clone, Default)]
    pub struct State {
        pub user_name: Option<UserName>,
    }
}
