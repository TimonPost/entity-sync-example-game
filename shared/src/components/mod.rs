use legion_sync::tracking::*;
use net_sync::transport::ClientId;

#[sync]
#[derive(Debug)]
pub struct Position {
    pub x: u16,
    pub y: u16
}

impl Position {
    pub fn new(x: u16, y: u16) -> Self {
        Position {
            x,
            y
        }
    }

    pub fn set(&mut self, pos: (u16,u16)) {
        self.x = pos.0;
        self.y = pos.1;
    }
}

impl Default for Position {
    fn default() -> Self {
        Position {
            x: 0,
            y: 0
        }
    }
}

#[sync]
#[derive(Debug)]
pub struct PlayerType {
   pub player_type: u16
}

impl PlayerType {
    pub fn new(player_type: PlayerTypeOp) -> PlayerType {
        PlayerType {
            player_type: player_type as u16
        }
    }

    pub fn player_type(&self) -> PlayerTypeOp {
        match self.player_type {
            1 => PlayerTypeOp::Enemy,
            2 => PlayerTypeOp::Player,
            _ => panic!("Player type not supported")
        }
    }
}

#[derive(Debug)]
#[derive(PartialOrd, PartialEq)]
pub enum  PlayerTypeOp {
    Enemy = 1,
    Player = 2
}

impl Default for PlayerType {
    fn default() -> Self {
        PlayerType {
            player_type: 1
        }
    }
}

#[sync]
#[derive(Debug,PartialOrd, PartialEq)]
pub struct PlayerInfo {
    client_id: ClientId,
}

impl PlayerInfo {
    pub fn new(client_id: ClientId) -> PlayerInfo {
        PlayerInfo {
            client_id
        }
    }

    pub fn client_id(&self) -> ClientId {
        self.client_id
    }
}


impl Default for PlayerInfo {
    fn default() -> Self {
        PlayerInfo {
            client_id: 9999
        }
    }
}


