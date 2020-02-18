use legion::prelude::{IntoQuery, Schedulable, SystemBuilder, Read};
use std::io::{stdout, Write};
use crossterm::QueueableCommand;
use crossterm::terminal::{ClearType, Clear};
use crossterm::cursor::MoveTo;
use crossterm::style::Print;
use legion_sync::components::UuidComponent;

pub fn draw_player_system() -> Box<dyn Schedulable> {
    SystemBuilder::new("draw_player_system")
        .with_query(<(legion::prelude::Write<crate::components::Position>, Read<UuidComponent>)>::query())
        .build(|_, mut world, _, query| {
            for (pos, _) in query.iter_mut(&mut world) {
                let mut stdout = stdout();
                stdout.queue(Clear(ClearType::All));
                stdout.queue(MoveTo(pos.x, pos.y));
                stdout.queue(Print("X"));
                stdout.flush();
            }
        })
}