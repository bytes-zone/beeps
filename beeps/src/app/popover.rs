use super::auth_form;
use chrono::{DateTime, Utc};
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Style, Stylize},
    widgets::{Block, Borders, Clear, Padding, Paragraph, Row, Table},
    Frame,
};
use tui_input::Input;

/// States shown above the main table.
#[derive(Debug)]
pub enum Popover {
    /// Show a table of keyboard shortcuts
    Help,

    /// Editing the tag for a ping
    Editing(DateTime<Utc>, Input),

    /// Register with the server
    Registering(auth_form::AuthForm),
}

impl Popover {
    /// Render the editing popover
    #[expect(clippy::cast_possible_truncation)]
    pub fn render(&mut self, frame: &mut Frame<'_>, body_area: Rect) {
        match self {
            Popover::Help => {
                let popup_vert = Layout::vertical([Constraint::Percentage(50)]).flex(Flex::Center);
                let popup_horiz =
                    Layout::horizontal([Constraint::Percentage(50)]).flex(Flex::Center);

                let [popup_area] = popup_vert.areas(body_area);
                let [popup_area] = popup_horiz.areas(popup_area);

                let popup = Table::new(
                    [
                        Row::new(vec!["? / F1", "Display this help"]),
                        Row::new(vec!["j / down", "Select ping below"]),
                        Row::new(vec!["k / up", "Select ping above"]),
                        Row::new(vec!["e / enter", "Edit tag for selected ping"]),
                        Row::new(vec!["c", "Copy tag for selected ping"]),
                        Row::new(vec!["v", "Paste copied tag to selected ping"]),
                        Row::new(vec!["q", "Quit / Close help"]),
                        Row::new(vec!["r", "Register a new account with the server"]),
                        Row::new(vec!["enter (editing)", "Save"]),
                        Row::new(vec!["escape (editing)", "Cancel"]),
                    ],
                    [Constraint::Max(16), Constraint::Fill(1)],
                )
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Keyboard Shortcuts")
                        .padding(Padding::horizontal(1))
                        .border_style(Style::new().blue()),
                );

                frame.render_widget(Clear, popup_area);
                frame.render_widget(popup, popup_area);
            }
            Popover::Editing(ping, tag_input) => {
                let popup_vert = Layout::vertical([Constraint::Length(3)]).flex(Flex::Center);
                let popup_horiz =
                    Layout::horizontal([Constraint::Percentage(50)]).flex(Flex::Center);

                let [popup_area] = popup_vert.areas(body_area);
                let [popup_area] = popup_horiz.areas(popup_area);

                let width = popup_area.width - 2 - 1; // -2 for the border, -1 for the cursor

                let input_scroll = tag_input.visual_scroll(width as usize);

                let popup = Paragraph::new(tag_input.value())
                    .scroll((0, input_scroll as u16))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(format!("Edit tag for {}", ping.to_rfc2822())),
                    )
                    .style(Style::default().blue());

                frame.render_widget(Clear, popup_area);
                frame.render_widget(popup, popup_area);

                frame.set_cursor_position((
                    popup_area.x
                                + (tag_input.visual_cursor().max(input_scroll) - input_scroll) as u16 // current end of text
                                + 1, // just past the end of the text
                    popup_area.y + 1, // +1 row for the border/title
                ));
            }
            Popover::Registering(auth) => auth.render(body_area, frame),
        }
    }
}
