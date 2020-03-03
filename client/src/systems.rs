use crossterm::style::Color;
use crossterm::{
    event,
    event::{Event, KeyCode, KeyEvent},
    QueueableCommand,
};
use legion::prelude::{IntoQuery, *};
use legion_sync::{components::UidComponent, resources::EventResource};
use log::debug;
use net_sync::uid::UidAllocator;
use shared::components::{Coloring, PlayerType, PlayerTypeOp, Position};
use std::time::Duration;
use track::Trackable;

pub fn move_player_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("move player")
        .read_resource::<EventResource>()
        .with_query(<(
            legion::prelude::Write<Position>,
            Read<UidComponent>,
            Read<PlayerType>,
        )>::query())
        .build(|_, mut world, resource, query| {
            let new_pos = |event, x, y| -> (u16, u16) {
                match event {
                    Event::Key(KeyEvent {
                        code: KeyCode::Up, ..
                    }) => (x, y - 1),
                    Event::Key(KeyEvent {
                        code: KeyCode::Right,
                        ..
                    }) => (x + 1, y),
                    Event::Key(KeyEvent {
                        code: KeyCode::Down,
                        ..
                    }) => (x, y + 1),
                    Event::Key(KeyEvent {
                        code: KeyCode::Left,
                        ..
                    }) => (x - 1, y),
                    _ => (x, y),
                }
            };

            if event::poll(Duration::from_millis(0)).unwrap() {
                let event = event::read().unwrap();

                for (mut pos, identifier, player) in query.iter_mut(&mut world) {
                    if player.player_type() == PlayerTypeOp::Player {
                        let mut pos = pos.track(resource.notifier(), identifier.uid());
                        let new_pos = new_pos(event, pos.x, pos.y);
                        pos.set(new_pos);
                    }
                }
            };
        })
}

pub fn insert_enemy_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("move player")
        .write_resource::<ClientState>()
        .build(|command, _, resource, _| {
            if resource.frame % 100 == 0 {
                command.insert(
                    (),
                    vec![(
                        UidComponent::new(resource.uid_allocator.allocate(None)),
                        Position::new(10, 15),
                        Coloring::new(Color::Red),
                        PlayerType::new(PlayerTypeOp::Enemy),
                    )],
                );
            }
        })
}

pub fn move_enemy_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("move player")
        .write_resource::<ClientState>()
        .read_resource::<EventResource>()
        .with_query(<(
            Read<UidComponent>,
            legion::prelude::Write<Position>,
            Read<Coloring>,
            Read<PlayerType>,
        )>::query())
        .build(|_, mut world, resource, query| {
            if resource.0.frame % 25 == 0 {
                for (id, mut pos, _, player) in query.iter_mut(&mut world) {
                    if player.player_type() == PlayerTypeOp::Enemy {
                        let mut pos = pos.track(resource.1.notifier(), id.uid());

                        if pos.x == 0 {
                            pos.x = 40;
                        } else {
                            pos.x -= 1;
                        }
                    }
                }
            }
            resource.0.frame += 1;
        })
}

pub struct ClientState {
    pub uid_allocator: UidAllocator,
    pub frame: u32,
}

impl ClientState {
    pub fn new(alloc: UidAllocator) -> ClientState {
        ClientState {
            uid_allocator: alloc,
            frame: 0,
        }
    }
}
