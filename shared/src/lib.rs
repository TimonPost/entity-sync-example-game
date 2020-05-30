use crate::message::ClientCommand;
use net_sync::transport::ClientId;
use sdl2::rect::Rect;
use sdl2::render::{TextureCreator, Texture};
use sdl2::video::WindowContext;
use sdl2::ttf::Font;
use sdl2::pixels::Color;

pub mod components;
pub mod systems;
pub mod message;

pub static LATENCY: u32 = 200;
pub static COMMAND_FRAME_INTERVAL: u32 = 1000;
pub static FRAME_INTERVAL: u32 = 200;
pub static MOVE_VELOCITY: u16 = 20;

pub const SCREEN_WIDTH: u32 = 800;
pub const SCREEN_HEIGHT: u32 = 500;

pub fn calculate_player_movement(event: &ClientCommand, x: u16, y: u16) -> (u16, u16) {
    match event {
        &ClientCommand::MoveUp => (x, y - (1 * MOVE_VELOCITY)),
        &ClientCommand::MoveRight => (x + (1 * MOVE_VELOCITY), y),
        &ClientCommand::MoveDown => (x, y + (1 * MOVE_VELOCITY)),
        &ClientCommand::MoveLeft => (x - (1 * MOVE_VELOCITY), y),
    }
}

#[derive(PartialOrd, PartialEq)]
pub enum ConnectionState {
    Connected,
    Connecting,
    Disconnected,
    Disconnecting
}

pub struct ConnectionInformation {
    state: ConnectionState,
    client_id: Option<ClientId>,
}

impl ConnectionInformation {
    pub fn new() -> ConnectionInformation {
        ConnectionInformation {
            state: ConnectionState::Disconnected,
            client_id: None,
        }
    }

    pub fn set_connected(&mut self, client_id: ClientId) {
        self.state = ConnectionState::Connected;
        self.client_id = Some(client_id);
    }

    pub fn set_disconnected(&mut self) {
        self.state = ConnectionState::Disconnected;
    }

    pub fn set_disconnecting(&mut self) {
        self.state = ConnectionState::Disconnecting;
    }

    pub fn set_connecting(&mut self) {
        self.state = ConnectionState::Connecting;
    }

    pub fn connection_state(&self) -> &ConnectionState {
        &self.state
    }

    pub fn is_connected(&self) -> bool {
        self.state == ConnectionState::Connected
    }

    pub fn client_id(&self) -> ClientId {
        self.client_id.expect("Tried to get client ID while the client is not yet connected.")
    }
}

pub fn create_texture_from_text<'a>(texture_creator: &'a TextureCreator<WindowContext>, font: &Font, text: &str, r: u8, g: u8, b: u8) -> Texture<'a> {
    if let Ok(surface) = font.render(text).blended(Color::RGB(r, g, b)) {
        texture_creator.create_texture_from_surface(&surface)
            .map_err(|e| e.to_string()).unwrap()
    }else {
        panic!("failed to create texture");
    }
}

pub fn get_rect_from_text(text: &str, x: i32, y: i32) -> Rect {
    Rect::new(x, y, text.len() as u32 * 20, 30)
}