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
pub type MaybeMsgId = Option<MsgId>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum WsClientMsg {
    UserName(String),
}

impl WsMsg<WsClientMsg> {
    pub fn ack(&self) -> Option<WsMsg<WsServerMsg>> {
        self.0
            .as_ref()
            .map(|id| WsMsg(Some(id.clone()), WsServerMsg::Ack))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum WsServerMsg {
    Ack,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WsMsg<M>(
    pub MaybeMsgId, /* may be used as acknowledgement ID / idempotency key */
    pub M,
);

impl<M> WsMsg<M> {
    pub fn new(msg: M) -> Self {
        Self(None, msg)
    }

    pub fn id(self, id: impl Into<MsgId>) -> Self {
        Self(Some(id.into()), self.1)
    }
}
