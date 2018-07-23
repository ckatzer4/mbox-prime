use std::io;
use std::sync::mpsc;
use std::thread;

use email::MimeMessage;
use email::mimeheaders::MimeContentTypeHeader;

use termion::event;
use termion::input::TermRead;

use tui::backend::MouseBackend;
use tui::layout::{Direction, Group, Rect, Size};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Paragraph, SelectableList, Widget};
use tui::Terminal;

enum Side {
    Names,
    Email,
}

pub struct App {
    size: Rect,
    names: Vec<String>,
    messages: Vec<MimeMessage>,
    selected: usize,
    active: Side,
    offset: u16,
}

impl App {
    pub fn new(names: Vec<String>, messages: Vec<MimeMessage>) -> App {
        App{
            size: Rect::default(),
            names: names,
            messages: messages,
            selected: 0,
            active: Side::Names,
            offset: 0,
        }
    }

    /// jump to the email the current selected message is replying to 
    fn find_parent(&mut self) {
        let msg = &self.messages[self.selected];

        let target_id: Option<String> = msg.headers
            .get("In-Reply-To".to_string())
            .and_then(|h| h.get_value().ok());

        // Look up the message with the target id
        if let Some(id) = target_id {
            let target = &self.messages.iter().position(
                |m| m.headers
                    .get("Message-ID".to_string())
                    .and_then(|h| h.get_value().ok())
                    .map(|i: String| i == id)
                    .unwrap_or(false)
            );

            if let Some(target_index) = target {
                self.selected = *target_index;
                self.offset = 0;
            }
        }
    }

    /// jump to the first reply to the current selected message
    fn find_child(&mut self) {
        let msg = &self.messages[self.selected];

        let target_id: Option<String> = msg.headers
            .get("Message-ID".to_string())
            .and_then(|h| h.get_value().ok());

        // Look up the message with the target id
        if let Some(id) = target_id {
            let target = &self.messages[self.selected..].iter().position(
                |m| m.headers
                    .get("In-Reply-To".to_string())
                    .and_then(|h| h.get_value().ok())
                    .map(|i: String| i == id)
                    .unwrap_or(false)
            );

            if let Some(target_index) = target {
                self.selected += *target_index;
                self.offset = 0;
            }
        }
    }

    /// jumpt to the previous email that shares a parent with the current selected message
    fn prev_sibling(&mut self) {
        let msg = &self.messages[self.selected];

        let target_id: Option<String> = msg.headers
            .get("In-Reply-To".to_string())
            .and_then(|h| h.get_value().ok());

        // Look up the message with the target id
        if let Some(id) = target_id {
            let target = &self.messages[0..self.selected].iter().position(
                |m| m.headers
                    .get("In-Reply-To".to_string())
                    .and_then(|h| h.get_value().ok())
                    .map(|i: String| i == id)
                    .unwrap_or(false)
            );

            if let Some(target_index) = target {
                self.selected = *target_index;
                self.offset = 0;
            }
        }
    }

    /// jumpt to the next email that shares a parent with the current selected message
    fn next_sibling(&mut self) {
        let msg = &self.messages[self.selected];

        let target_id: Option<String> = msg.headers
            .get("In-Reply-To".to_string())
            .and_then(|h| h.get_value().ok());

        // Look up the message with the target id
        if let Some(id) = target_id {
            let target = &self.messages[self.selected+1..].iter().position(
                |m| m.headers
                    .get("In-Reply-To".to_string())
                    .and_then(|h| h.get_value().ok())
                    .map(|i: String| i == id)
                    .unwrap_or(false)
            );

            if let Some(target_index) = target {
                self.selected += target_index+1;
                self.offset = 0;
            }
        }
    }

    pub fn run(&mut self) {
        // Terminal initialization
        let backend = MouseBackend::new().unwrap();
        let mut terminal = Terminal::new(backend).unwrap();

        // Channels
        let (tx, rx) = mpsc::channel();
        let input_tx = tx.clone();

        // Input
        thread::spawn(move || {
            let stdin = io::stdin();
            for c in stdin.keys() {
                let evt = c.unwrap();
                input_tx.send(evt).unwrap();
                if evt == event::Key::Char('q') {
                    break;
                }
            }
        });

        // First draw call
        terminal.clear().unwrap();
        terminal.hide_cursor().unwrap();
        self.size = terminal.size().unwrap();
        draw(&mut terminal, &self);

        loop {
            let size = terminal.size().unwrap();
            if size != self.size {
                terminal.resize(size).unwrap();
                self.size = size;
            }

            let input = rx.recv().unwrap();
            match input {
                event::Key::Char('q') => {
                    break;
                }
                event::Key::Char('h') => {
                    self.find_parent();
                }
                event::Key::Char('l') => {
                    self.find_child();
                }
                event::Key::Char('N') => {
                    self.prev_sibling();
                }
                event::Key::Char('n') => {
                    self.next_sibling();
                }
                event::Key::Char('\t') => {
                    match self.active {
                        Side::Names => {
                            self.active = Side::Email;
                        }
                        Side::Email => {
                            self.active = Side::Names;
                        }
                    }
                }
                event::Key::Down | event::Key::Char('j') => {
                    match self.active {
                        Side::Names => {
                            if self.selected < self.names.len() - 1 {
                                self.selected += 1;
                            }
                            self.offset = 0;
                        }
                        Side::Email => {
                            self.offset += 1;
                        }
                    }
                }
                event::Key::Up | event::Key::Char('k') => {
                    match self.active {
                        Side::Names => {
                            if self.selected > 0 {
                                self.selected -= 1;
                            }
                        }
                        Side::Email => {
                            if self.offset >0 {
                                self.offset -= 1;
                            }
                        }
                    }
                }
                event::Key::PageUp => {
                    let jump = (self.size.height - 2) as usize;
                    match self.active {
                        Side::Names => {
                            if self.selected > jump {
                                self.selected -= jump;
                            } else {
                                self.selected = 0;
                            }
                        }
                        Side::Email => {
                            // Ideally, we should check for the length of the email first
                            // But, at least this prevents overflow
                            self.offset = self.offset.saturating_sub(jump as u16);
                        }
                    }
                }
                event::Key::PageDown => {
                    let jump = (self.size.height - 2) as usize;
                    match self.active {
                        Side::Names => {
                            if self.selected < self.names.len() - jump {
                                self.selected += jump;
                            } else {
                                self.selected = self.names.len() - 1;
                            }
                        }
                        Side::Email => {
                            // Ideally, we should check for the length of the email first
                            // But, at least this prevents overflow
                            self.offset = self.offset.saturating_add(jump as u16);
                        }
                    }
                }
                _ => {}
            };

            terminal.draw().unwrap();
            draw(&mut terminal, &self);
        }
        terminal.clear().unwrap();
        terminal.show_cursor().unwrap();
    }

}

fn display_message(email: &MimeMessage) -> String {
    let mut output = String::new();
    if let Some(header) = email.headers.get("From".into()) {
        output.push_str(&format!("{}\n", header));
    }
    if let Some(header) = email.headers.get("Date".into()) {
        output.push_str(&format!("{}\n", header));
    }
    if let Some(header) = email.headers.get("Content-Type".into()) {
        let content_type: MimeContentTypeHeader = header.get_value().unwrap();
        let (mime_type, sub_mime_type) = content_type.content_type;
        output.push_str(&format!("Content-Type: {}/{}\n", mime_type, sub_mime_type));
        // let boundary = content_type.params.get("boundary");
        // output.push_str(&format!("Boundary: {:?}\n", boundary));

    }
    // output.push_str(&format!("Children: {}\n", email.children.len()));
    output.push('\n');

    match email.decoded_body_string() {
        Ok(body) => {
            output.push_str(&body);
        }
        Err(e) => {
            let error_body = format!("Error decoding body: {:?}", e);
            output.push_str(&error_body);
        }
    }
    for child in email.children.iter() {
        output.push_str(&display_message(child));
    }
    output
}

fn draw(t: &mut Terminal<MouseBackend>, app: &App) {
    Group::default()
        .direction(Direction::Horizontal)
        .sizes(
            // Give more room to the active side
            match app.active {
                Side::Names => {
                    &[Size::Percent(70), Size::Percent(30)]
                }
                Side::Email => {
                    &[Size::Percent(30), Size::Percent(70)]
                }
            }
        )
        .render(t, &app.size, |t, chunks| {
            let style = Style::default();
            SelectableList::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("List")
                )
                .items(&app.names)
                .select(app.selected)
                .style(style)
                .highlight_style(
                    style.clone()
                        .fg(Color::LightGreen)
                )
                .highlight_symbol(">")
                .render(t, &chunks[0]);
            Paragraph::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Email")
                )
                .wrap(true)
                .text(
                    &display_message(&app.messages[app.selected])
                )
                .scroll(app.offset)
                .render(t, &chunks[1]);
        });

    t.draw().unwrap();
}

