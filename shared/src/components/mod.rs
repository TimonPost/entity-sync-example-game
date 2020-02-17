use track::preclude::*;

#[track]
#[derive(Debug)]
pub struct Position {
    pub x: u16,
    pub y: u16
}

impl Position {
    pub fn set(&mut self, pos: (u16,u16)) {
        self.x = pos.0;
        self.y = pos.1;
    }
}


