use legion::prelude::{IntoQuery, Schedulable, SystemBuilder, Read};
use std::io::{stdout, Write};
use crossterm::QueueableCommand;
use crossterm::terminal::{ClearType, Clear};
use crossterm::cursor::MoveTo;
use crossterm::style::Print;
use legion_sync::components::UidComponent;
use crate::components::Position;

pub fn draw_player_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("draw_player_system")
        .with_query(<(Read<UidComponent>, legion::prelude::Read<Position>)>::query())
        .build(|_, mut world, _, query| {
            for (_, pos) in query.iter_mut(&mut world) {
                let mut stdout = stdout();
                stdout.queue(Clear(ClearType::All));
                stdout.queue(MoveTo(pos.x, pos.y));
                stdout.queue(Print("X"));
                stdout.flush();
            }
        })
}

pub fn draw_entities() -> Box<dyn Schedulable> {
    SystemBuilder::new("draw_player_system")
        .with_query(<(Read<UidComponent>, legion::prelude::Read<Position>)>::query())
        .build(|_, mut world, _, query| {
            for (uid, pos) in query.iter_mut(&mut world) {
                let mut stdout = stdout();
                stdout.queue(Clear(ClearType::All));
                stdout.queue(MoveTo(0, 0));
                stdout.queue(Print(format!("{:?} {:?}\r\n", *uid, *pos)));
                stdout.flush();
            }
        })
}