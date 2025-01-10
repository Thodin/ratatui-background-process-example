use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout},
    prelude::{Buffer, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Gauge, Paragraph, Widget},
    DefaultTerminal, Frame,
};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App {
        exit: false,
        background_progress: 0.3134_f64,
    };
    let app_result = app.run(&mut terminal);
    ratatui::restore();
    app_result
}

pub struct App {
    exit: bool,
    background_progress: f64,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                self.exit = true;
            }
        }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let horizontal_layout =
            Layout::vertical([Constraint::Percentage(20), Constraint::Percentage(80)]);
        let [text_area, gauge_area] = horizontal_layout.areas(area);
        let block = Block::bordered()
            .title(Line::from(" Background processes "))
            .border_set(border::THICK);
        let line = Line::from("Process overview").bold();
        // Span::raw("Test").yellow().render(area, buf);
        let progress_bar = Gauge::default()
            .gauge_style(Style::default().fg(Color::Green))
            .block(block)
            .label(format!("Process 1: {:.2}%", self.background_progress))
            .ratio(self.background_progress);

        Paragraph::new(line).render(text_area, buf);
        progress_bar.render(
            Rect {
                x: gauge_area.left(),
                y: gauge_area.top(),
                width: gauge_area.width,
                height: 3,
            },
            buf,
        );
    }
}
