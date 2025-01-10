use std::{io, sync::mpsc, thread, time::Duration};

use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout},
    prelude::{Buffer, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Gauge, Widget},
    DefaultTerminal, Frame,
};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();

    // Create the channel via which the events will be sent to the main app.
    let (event_tx, event_rx) = mpsc::channel::<Event>();

    // Thread to listen for input events.
    let tx_to_input_events = event_tx.clone();
    thread::spawn(move || {
        handle_input_events(tx_to_input_events);
    });

    // Thread that does a computational heavy task.
    let tx_to_background_progress_events = event_tx.clone();
    thread::spawn(move || {
        run_background_thread(tx_to_background_progress_events);
    });

    let mut app = App {
        exit: false,
        progress_bar_color: Color::Green,
        background_progress: 0_f64,
    };

    // App runs on the main thread.
    let app_result = app.run(&mut terminal, event_rx);

    // Note: If your threads need clean-up (i.e. the computation thread),
    // you should communicatie to them that the app wants to shut down.
    // This is not required here, as our threads don't use resources.
    ratatui::restore();
    app_result
}

// Events that can be sent to the main thread.
enum Event {
    Input(event::KeyEvent), // crossterm key input event
    Resize,                 // Resize event
    Progress(f64),          // progress update from the computation thread
}

pub struct App {
    exit: bool,
    progress_bar_color: Color,
    background_progress: f64,
}

/// Block, waiting for input events from the user.
fn handle_input_events(tx: mpsc::Sender<Event>) {
    loop {
        match event::read().unwrap() {
            event::Event::Key(key_event) => tx.send(Event::Input(key_event)).unwrap(),
            event::Event::Resize(_, _) => tx.send(Event::Resize).unwrap(),
            _ => {}
        }
    }
}

/// Simulate a computational heavy task.
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
    /// Main task to be run continuously
    fn run(&mut self, terminal: &mut DefaultTerminal, rx: mpsc::Receiver<Event>) -> io::Result<()> {
        while !self.exit {
            match rx.recv().unwrap() {
                Event::Input(key_event) => self.handle_key_event(key_event)?,
                Event::Progress(progress) => self.background_progress = progress,
                Event::Resize => terminal.autoresize()?,
            }
            terminal.draw(|frame| self.draw(frame))?;
        }
        Ok(())
    }

    /// Render `self`, as we implemented the Widget trait for &App
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    /// Actions that should be taken when a key event comes in.
    fn handle_key_event(&mut self, key_event: event::KeyEvent) -> io::Result<()> {
        if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Char('q') {
            self.exit = true;
        } else if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Char('c') {
            if self.progress_bar_color == Color::Green {
                self.progress_bar_color = Color::Yellow;
            } else {
                self.progress_bar_color = Color::Green;
            }
        }

        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Split the screen vertically in a 20:80 ratio.
        // Top is used for title, bottom for the progress gauge.
        let vertical_layout =
            Layout::vertical([Constraint::Percentage(20), Constraint::Percentage(80)]);
        let [title_area, gauge_area] = vertical_layout.areas(area);

        // Render a title in the top of the layout
        Line::from("Process overview")
            .bold()
            .render(title_area, buf);

        // Prepare the widgets for the bottom part of the layout.
        let instructions = Line::from(vec![
            " Change color ".into(),
            "<C>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ])
        .centered();

        // Block to be displayed around the progress bar.
        let block = Block::bordered()
            .title(Line::from(" Background processes "))
            .title_bottom(instructions)
            .border_set(border::THICK);

        // Progress bar with label on it.
        let progress_bar = Gauge::default()
            .gauge_style(Style::default().fg(self.progress_bar_color))
            .block(block)
            .label(format!(
                "Process 1: {:.2}%",
                self.background_progress * 100_f64
            ))
            .ratio(self.background_progress);

        // Render the progress bar in the gauge area, with a fixed height of 3 lines (2 for block, 1 for bar)
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
