use ratatui::layout::Constraint;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Row, Table, TableState};
use ratatui::Frame;
use ratatui::layout::Rect;

use crate::app::{App, Panel};
use crate::ui::layout::body_split;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let chunks = body_split(area);

    render_playlists(f, app, chunks[0]);
    render_tracks(f, app, chunks[1]);
}

fn render_playlists(f: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::Left;
    let border_style = if is_active {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let items: Vec<ListItem> = app
        .playlists
        .iter()
        .enumerate()
        .map(|(i, playlist)| {
            let style = if i == app.playlist_index && is_active {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(playlist.name.as_str()).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(" Playlists "),
    );

    let mut state = ListState::default();
    state.select(Some(app.playlist_index));
    f.render_stateful_widget(list, area, &mut state);
}

fn render_tracks(f: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::Right;
    let border_style = if is_active {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    if app.loading && app.playlist_tracks.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(" Tracks ");
        let loading = ratatui::widgets::Paragraph::new("Loading...")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        f.render_widget(loading, area);
        return;
    }

    let header = Row::new(vec!["#", "Title", "Artist", "Duration"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .playlist_tracks
        .iter()
        .enumerate()
        .map(|(i, track)| {
            let artists: Vec<&str> = track.artists.iter().map(|a| a.name.as_str()).collect();
            let duration_secs = track.duration.num_seconds();
            let duration = format!("{}:{:02}", duration_secs / 60, duration_secs % 60);
            let liked = if app.liked_track_ids.contains(
                &track.id.as_ref().map(|id| id.to_string()).unwrap_or_default(),
            ) {
                "♥"
            } else {
                ""
            };
            let style = if i == app.track_index && is_active {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            Row::new(vec![
                format!("{} {}", i + 1, liked),
                track.name.clone(),
                artists.join(", "),
                duration,
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(6),
            Constraint::Percentage(45),
            Constraint::Percentage(35),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(" Tracks "),
    )
    .row_highlight_style(
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    );

    let mut state = TableState::default();
    state.select(Some(app.track_index));
    f.render_stateful_widget(table, area, &mut state);
}
