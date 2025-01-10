use std::{io, sync::mpsc, thread, time::Duration};

use crossterm::event::{self, KeyCode, KeyEventKind};
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
    let (event_tx, event_rx) = mpsc::channel::<Event>();

    let tx_to_input_events = event_tx.clone();
    thread::spawn(move || {
        handle_input_events(tx_to_input_events);
    });

    let tx_to_tick_events = event_tx.clone();
    thread::spawn(move || {
        tick_event(tx_to_tick_events);
    });

    let tx_to_background_progress_events = event_tx.clone();
    thread::spawn(move || {
        run_background_thread(tx_to_background_progress_events);
    });

    let mut app = App {
        exit: false,
        background_progress: 0_f64,
    };

    let app_result = app.run(&mut terminal, event_rx);
    ratatui::restore();
    app_result
}

enum Event {
    Input(event::KeyEvent), // crossterm event
    Progress(f64),
    Tick,
    Resize,
}

pub struct App {
    exit: bool,
    background_progress: f64,
}

fn handle_input_events(tx: mpsc::Sender<Event>) {
    loop {
        match event::read().unwrap() {
            event::Event::Key(key_event) => tx.send(Event::Input(key_event)).unwrap(),
            event::Event::Resize(_, _) => tx.send(Event::Resize).unwrap(),
            _ => {}
        }
    }
}

fn tick_event(tx: mpsc::Sender<Event>) {
    loop {
        thread::sleep(Duration::from_millis(200));
        tx.send(Event::Tick).unwrap();
    }
}

fn run_background_thread(tx: mpsc::Sender<Event>) {
    let mut progress = 0_f64;
    let increment = 0.01_f64;
    loop {
        thread::sleep(Duration::from_millis(100));
        progress += increment;
        progress = progress.min(1_f64);
        tx.send(Event::Progress(progress)).unwrap();
    }
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal, rx: mpsc::Receiver<Event>) -> io::Result<()> {
        while !self.exit {
            match rx.recv().unwrap() {
                Event::Input(key_event) => self.handle_key_event(key_event)?,
                Event::Progress(progress) => self.background_progress = progress,
                Event::Tick => {} // don't need to do anything here, tick is only used to trigger a re-draw.
                Event::Resize => terminal.autoresize()?,
            }
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_key_event(&mut self, key_event: event::KeyEvent) -> io::Result<()> {
        if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Char('q') {
            self.exit = true;
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
        let progress_bar = Gauge::default()
            .gauge_style(Style::default().fg(Color::Green))
            .block(block)
            .label(format!(
                "Process 1: {:.2}%",
                self.background_progress * 100_f64
            ))
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
