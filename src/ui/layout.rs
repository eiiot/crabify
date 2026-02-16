use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Tabs};
use ratatui::Frame;

use crate::app::{App, Screen};

pub fn main_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header (tabs)
            Constraint::Min(8),   // Body
            Constraint::Length(3), // Footer (now playing)
        ])
        .split(area)
        .to_vec()
}

pub fn body_split(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Left panel
            Constraint::Percentage(70), // Right panel
        ])
        .split(area)
        .to_vec()
}

pub fn render_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = Screen::all()
        .iter()
        .map(|s| {
            let style = if *s == app.screen {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            Line::from(Span::styled(s.label(), style))
        })
        .collect();

    let selected = Screen::all()
        .iter()
        .position(|s| *s == app.screen)
        .unwrap_or(0);

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 🦀 crabify "),
        )
        .select(selected)
        .highlight_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::raw(" | "));

    f.render_widget(tabs, area);
}

pub fn render_flash(f: &mut Frame, msg: &str) {
    let area = f.area();
    let popup_width = (msg.len() as u16 + 4).min(area.width - 4);
    let popup_area = Rect {
        x: area.width.saturating_sub(popup_width) / 2,
        y: area.height.saturating_sub(5) / 2,
        width: popup_width,
        height: 3,
    };

    f.render_widget(Clear, popup_area);
    let paragraph = Paragraph::new(msg.to_string())
        .style(Style::default().fg(Color::Red))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red))
                .title(" Error "),
        );
    f.render_widget(paragraph, popup_area);
}
