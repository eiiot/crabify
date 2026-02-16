use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table, TableState};
use ratatui::Frame;

use crate::app::{App, InputMode};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input
            Constraint::Min(5),   // Results
        ])
        .split(area);

    render_search_input(f, app, chunks[0]);
    render_results(f, app, chunks[1]);
}

fn render_search_input(f: &mut Frame, app: &App, area: Rect) {
    let (border_color, title) = if app.input_mode == InputMode::Editing {
        (Color::Green, " Search (type and press Enter) ")
    } else {
        (Color::DarkGray, " Search (press / to start) ")
    };

    let input = Paragraph::new(app.search_input.as_str())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(title),
        );
    f.render_widget(input, area);

    // Show cursor when editing
    if app.input_mode == InputMode::Editing {
        f.set_cursor_position((
            area.x + app.search_input.len() as u16 + 1,
            area.y + 1,
        ));
    }
}

fn render_results(f: &mut Frame, app: &App, area: Rect) {
    if app.loading && app.search_results.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Results ");
        let loading = Paragraph::new("Searching...")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        f.render_widget(loading, area);
        return;
    }

    if app.search_results.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Results ");
        let empty = Paragraph::new(if app.search_input.is_empty() {
            "Press / to search for tracks"
        } else {
            "No results found"
        })
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
        .search_results
        .iter()
        .enumerate()
        .map(|(i, track)| {
            let artists: Vec<&str> = track.artists.iter().map(|a| a.name.as_str()).collect();
            let album = track.album.name.as_str();
            let duration_secs = track.duration.num_seconds();
            let duration = format!("{}:{:02}", duration_secs / 60, duration_secs % 60);
            let liked = if app.liked_track_ids.contains(
                &track.id.as_ref().map(|id| id.to_string()).unwrap_or_default(),
            ) {
                "♥"
            } else {
                ""
            };

            let style = if i == app.search_index {
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
            .title(" Results "),
    )
    .row_highlight_style(
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    );

    let mut state = TableState::default();
    state.select(Some(app.search_index));
    f.render_stateful_widget(table, area, &mut state);
}
