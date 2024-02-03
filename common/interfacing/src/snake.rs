use crate::imports::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct CreateLobby {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GetLobby {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct JoinLobbyAs {
    pub name: String,
}

pub type UserName = String;
pub type LobbyName = String;

pub type MsgId = String;
pub type MaybeMsgId = Option<MsgId>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum WsClientMsg {
    SetUserName(UserName),
    UserName,
    JoinLobby(LobbyName),
    LobbyList,
    VoteStart(bool),
}

impl WsMsg<WsClientMsg> {
    #[deprecated]
    pub fn ack(&self) -> Option<WsMsg<WsServerMsg>> {
        self.0
            .as_ref()
            .map(|id| WsMsg(Some(id.clone()), WsServerMsg::Ack))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum WsServerMsg {
    Ack,
    UserName(Option<UserName>),
    UserNameOccupied,
    ForbiddenWhenJoined,
    JoinLobbyDecline(JoinLobbyDecline),
    LobbyList(LobbyList),
    Err(String),
    LobbyChange(LobbyChange),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum JoinLobbyDecline {
    AlreadyJoined(LobbyName),
    NotFound,
    UserNameNotSet,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WsMsg<M>(
    pub MaybeMsgId, /* may be used as acknowledgement ID / idempotency key */
    pub M,
);

impl<M> WsMsg<M> {
    #[deprecated]
    pub fn new(msg: M) -> Self {
        Self(None, msg)
    }

    #[deprecated]
    pub fn id(self, id: impl Into<MsgId>) -> Self {
        Self(Some(id.into()), self.1)
    }

    #[deprecated]
    pub fn maybe_id(self, maybe_id: Option<impl Into<MsgId>>) -> Self {
        Self(maybe_id.map(Into::into), self.1)
    }
}

pub mod list {
    use crate::imports::*;

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    pub struct Lobby {
        pub name: String,
        // pub player_count: u32,
    }

    pub type LobbyList = Vec<Lobby>;
}

pub use list::LobbyList;

pub mod lobby_change {
    use crate::imports::*;

    use super::UserName;

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    pub enum LobbyChange {
        Prep(LobbyPrep),
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    pub struct LobbyPrep {
        pub participants: Vec<Participant>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    pub struct Participant {
        pub user_name: UserName,
        pub vote_start: bool,
    }
}

pub use lobby_change::LobbyChange;

pub trait PinnedMessage<T> {
    fn pinned_msg(self, msg: T) -> WsMsg<T>;
}

impl<T> PinnedMessage<T> for MsgId {
    fn pinned_msg(self, msg: T) -> WsMsg<T> {
        WsMsg(Some(self), msg)
    }
}

impl<T> PinnedMessage<T> for &str /* &MsgId does not work for &str */ {
    fn pinned_msg(self, msg: T) -> WsMsg<T> {
        self.to_owned().pinned_msg(msg)
    }
}
