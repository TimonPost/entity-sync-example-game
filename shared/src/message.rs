use net_sync::transport::{NetworkMessage, NetworkCommand, ClientId};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum ClientCommand {
    MoveUp,
    MoveRight,
    MoveDown,
    MoveLeft
}

#[derive(Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum ClientMessage {
    ConnectionRequest,
    Disconnect
}

#[derive(Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum ServerMessage {
    ConnectionAccepted(ClientId),
}

impl NetworkMessage for ClientMessage {}
impl NetworkMessage for ServerMessage {}
impl NetworkCommand for ClientCommand {}
impl NetworkMessage for ClientCommand {}