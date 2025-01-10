use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout},
    prelude::{Buffer, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{self, Line, Span},
    widgets::{Block, Gauge, LineGauge, Paragraph, Widget},
    DefaultTerminal, Frame,
};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App {
        exit: false,
        background_progress: 0.3_f64,
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
            .title(Line::from("Background process").centered().bold())
            .border_set(border::THICK);
        let line = Line::from("Test").yellow();
        // Span::raw("Test").yellow().render(area, buf);
        let progress_bar = Gauge::default()
            .gauge_style(Style::default().fg(Color::Green))
            .label("Progress")
            .ratio(self.background_progress);

        Paragraph::new(line)
            .centered()
            .block(block)
            .render(text_area, buf);
        progress_bar.render(gauge_area, buf);
    }
}
