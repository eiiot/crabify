use ratatui::layout::Constraint;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table, TableState};
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    if app.loading && app.liked_songs.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Liked Songs ");
        let loading = Paragraph::new("Loading...")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        f.render_widget(loading, area);
        return;
    }

    if app.liked_songs.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .title(" Liked Songs ");
        let empty = Paragraph::new("No liked songs found")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        f.render_widget(empty, area);
        return;
    }

    let header = Row::new(vec!["#", "Title", "Artist", "Album", "Duration"])
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .liked_songs
        .iter()
        .enumerate()
        .map(|(i, saved)| {
            let track = &saved.track;
            let artists: Vec<&str> = track.artists.iter().map(|a| a.name.as_str()).collect();
            let album = track.album.name.as_str();
            let duration_secs = track.duration.num_seconds();
            let duration = format!("{}:{:02}", duration_secs / 60, duration_secs % 60);

            let style = if i == app.liked_index {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            Row::new(vec![
                format!("♥ {}", i + 1),
                track.name.clone(),
                artists.join(", "),
                album.to_string(),
                duration,
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(6),
            Constraint::Percentage(30),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .title(" Liked Songs "),
    )
    .row_highlight_style(
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    );

    let mut state = TableState::default();
    state.select(Some(app.liked_index));
    f.render_stateful_widget(table, area, &mut state);
}
