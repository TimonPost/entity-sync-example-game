use net_sync::transport::{ClientId};
use serde::{Serialize, Deserialize};
use net_sync::synchronisation::{NetworkMessage, NetworkCommand};

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