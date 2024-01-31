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

pub type MsgId = String; // UUID

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum WsClientMsg {
    UserName(String),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum WsServerMsg {
    Ack,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Msg<M>(
    pub MsgId, /* may be used as acknowledgement ID / idempotency key */
    pub M,
);
