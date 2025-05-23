use crate::form_fields;
use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use tui_input::{backend::crossterm::EventHandler, Input};

/// A form for entering auth information
#[derive(Debug)]
pub struct AuthForm {
    /// Which field we're editing
    active: Field,

    /// What server to connect to
    server: Input,

    /// Who are you?
    email: Input,

    /// What's your password? (Will be masked)
    password: Input,
}

form_fields!(Field, Server, Email, Password);

impl AuthForm {
    /// Render this form to the screen
    #[expect(clippy::cast_possible_truncation)]
    pub fn render(&mut self, body_area: Rect, frame: &mut Frame<'_>) {
        let popup_vert = Layout::vertical([Constraint::Length(9)]).flex(Flex::Center);
        let popup_horiz = Layout::horizontal([Constraint::Percentage(50)]).flex(Flex::Center);

        let [popup_area] = popup_vert.areas(body_area);
        let [popup_area] = popup_horiz.areas(popup_area);
        frame.render_widget(Clear, popup_area);

        let width = popup_area.width - 2 - 1; // -2 for the border, -1 for the cursor

        let fields = Layout::vertical(Constraint::from_lengths([3, 3, 3]));
        let [server_area, email_area, password_area] = fields.areas(popup_area);

        let border_style = Style::default().fg(Color::Blue);

        // SERVER
        {
            let server_input_scroll = self.server.visual_scroll(width as usize);

            let server_field = Paragraph::new(self.server.value())
                .scroll((0, server_input_scroll as u16))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Server")
                        .border_style(border_style),
                );

            frame.render_widget(server_field, server_area);

            if matches!(self.active, Field::Server) {
                frame.set_cursor_position((
                    popup_area.x
                        + (self.server.visual_cursor().max(server_input_scroll) - server_input_scroll) as u16 // current end of text
                        + 1, // just past the end of the text
                    server_area.y + 1, // +1 row for the border/title
                ));
            }
        }

        // EMAIL
        {
            let email_input_scroll = self.email.visual_scroll(width as usize);

            let email_field = Paragraph::new(self.email.value())
                .scroll((0, email_input_scroll as u16))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Email")
                        .border_style(border_style),
                );

            frame.render_widget(email_field, email_area);

            if matches!(self.active, Field::Email) {
                frame.set_cursor_position((
                    popup_area.x
                        + (self.email.visual_cursor().max(email_input_scroll) - email_input_scroll) as u16 // current end of text
                        + 1, // just past the end of the text
                    email_area.y + 1, // +1 row for the border/title
                ));
            }
        }

        // PASSWORD
        {
            let password_input_scroll = self.password.visual_scroll(width as usize);

            let password_field = Paragraph::new("*".repeat(self.password.value().len()))
                .scroll((0, password_input_scroll as u16))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Password")
                        .border_style(border_style),
                );

            frame.render_widget(password_field, password_area);

            if matches!(self.active, Field::Password) {
                frame.set_cursor_position((
                    popup_area.x
                        + (self.password.visual_cursor().max(password_input_scroll) - password_input_scroll) as u16 // current end of text
                        + 1, // just past the end of the text
                    password_area.y + 1, // +1 row for the border/title
                ));
            }
        }
    }

    /// Handle a key event, updating the internal state of the form. This
    /// doesn't handle submission or cancellation events; it's up to the
    /// wrapping component to do that.
    pub fn handle_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Tab => {
                self.active = self.active.next();
            }
            KeyCode::BackTab => {
                self.active = self.active.prev();
            }
            _ => {
                let event = Event::Key(key);

                match self.active {
                    Field::Server => self.server.handle_event(&event),
                    Field::Email => self.email.handle_event(&event),
                    Field::Password => self.password.handle_event(&event),
                };
            }
        }
    }

    /// Once you're done filling out the form, call `finish` to unwrap the
    /// inputs into something you can use to make an HTTP request.
    pub fn finish(&self) -> AuthInfo {
        AuthInfo {
            server: self.server.to_string(),
            email: self.email.to_string(),
            password: self.password.to_string(),
        }
    }
}

impl Default for AuthForm {
    fn default() -> Self {
        Self {
            active: Field::Email,
            server: Input::new("https://beeps.bytes.zone".into()),
            email: Input::new(String::new()),
            password: Input::new(String::new()),
        }
    }
}

/// The output of using `Auth` to enter information
#[derive(Debug)]
pub struct AuthInfo {
    /// What server to connect to
    pub server: String,

    /// What username to use
    pub email: String,

    /// What password to use
    pub password: String,
}
