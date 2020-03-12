use legion_sync::tracking::*;
use crossterm::style::Color;

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
pub struct Coloring {
    color: u16
}

impl Coloring {
    pub fn new(color: Color) -> Self {
        let tag = match color {
            Color::Red => { 1},
            Color::Blue => { 2},
            _ => panic!("Color not supported.")
        };

        Coloring {
            color: tag
        }
    }

    pub fn color_type(&self) -> Color {
        match self.color {
            1 => Color::Red,
            2 => Color::Blue,
            _ => panic!("Color not supported")
        }
    }
}

impl Default for Coloring {
    fn default() -> Self {
        Coloring {
            color: 1
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


//#[track]
//#[derive(Debug)]
//struct PlayerInfo {
//    pub
//}


