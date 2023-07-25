use std::{
    collections::HashMap,
    io::{stdout, Write},
};

use crossterm::{
    self,
    cursor::MoveTo,
    event::{
        read, DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture,
        Event, MouseEvent, MouseEventKind,
    },
    style::Print,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, Clear, EnterAlternateScreen, LeaveAlternateScreen,
    },
    ExecutableCommand, QueueableCommand, Result,
};

struct Window {
    size: (u16, u16),
    position: (u16, u16),
    dots: HashMap<(u16, u16), char>,
}

impl Window {
    fn build(width: u16, height: u16, x: u16, y: u16) -> Self {
        let dots = (x..=x + width)
            .flat_map(|dot_x| (y..y + height).map(move |dot_y| ((dot_x, dot_y), '█')))
            .collect::<HashMap<(u16, u16), char>>();
        Self {
            size: (width, height),
            position: (x, y),
            dots,
        }
    }
    fn rebuild(&mut self) {
        *self = Self::build(self.size.0, self.size.1, self.position.0, self.position.1);
    }
    fn draw(&self) -> Result<()> {
        for ((x, y), element) in &self.dots {
            stdout().queue(MoveTo(*x, *y))?.queue(Print(element))?;
        }
        stdout().queue(MoveTo(0, 0))?;
        Ok(())
    }
    fn drog(&mut self, x: i8, y: i8) {
        self.position = (
            (self.position.0 as i16 + x as i16) as u16,
            (self.position.1 as i16 + y as i16) as u16,
        );
        self.rebuild();
    }
}
struct Mouse {
    position: (u16, u16),
}
impl Mouse {
    fn record(x: u16, y: u16) -> Self {
        Self { position: (x, y) }
    }
    fn update(&mut self, x: u16, y: u16, state: MouseEventKind) -> (i8, i8) {
        let result = match state {
            MouseEventKind::Drag(_) => (
                x as i8 - self.position.0 as i8,
                y as i8 - self.position.1 as i8,
            ),
            _ => (0, 0),
        };
        *self = Self::record(x, y);
        result
    }
}
fn main() -> Result<()> {
    enable_raw_mode()?;
    stdout()
        .queue(EnableMouseCapture)?
        .queue(EnableFocusChange)?
        .queue(EnterAlternateScreen)?;
    stdout().flush()?;
    let mut window = Window::build(20, 10, 10, 5);
    let mut mouse = Mouse::record(0, 0);
    window.draw()?;
    stdout().flush()?;
    loop {
        match read()? {
            Event::FocusGained => break,
            Event::FocusLost => break,
            Event::Key(_) => break,
            Event::Mouse(MouseEvent {
                kind,
                column,
                row,
                modifiers: _,
            }) => {
                if kind == MouseEventKind::Moved {
                    continue;
                }
                let (x, y) = mouse.update(column, row, kind);
                if window.position.0 < column
                    && column < window.position.0 + window.size.0
                    && window.position.1 < row
                    && row < window.position.1 + window.size.1
                {
                    stdout().queue(Clear(terminal::ClearType::All))?;
                    window.drog(x, y);
                    window.draw()?;
                    stdout().flush()?;
                }
            }
            Event::Paste(_) => todo!(),
            Event::Resize(x, y) => println!("X:{x}Y:{y}"),
        }
    }
    stdout().execute(LeaveAlternateScreen)?;
    stdout().execute(DisableMouseCapture)?;
    stdout().execute(DisableFocusChange)?;
    disable_raw_mode()?;
    Ok(())
}
