use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Now Playing ");

    if app.now_playing.is_none() {
        let empty = Paragraph::new("No playback detected. Start playing on a Spotify client.")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        f.render_widget(empty, area);
        return;
    }

    let inner = block.inner(area);
    f.render_widget(block, area);

    let track_name = app.current_track_name().unwrap_or_default();
    let progress_text = app.progress_text();
    let play_icon = if app.is_playing { "▶" } else { "⏸" };
    let volume_str = format!("Vol: {}%", app.volume);

    let line = Line::from(vec![
        Span::styled(
            format!(" {} ", play_icon),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            track_name,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(progress_text, Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled(volume_str, Style::default().fg(Color::DarkGray)),
    ]);

    f.render_widget(Paragraph::new(line), inner);
}
