use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

pub fn render(f: &mut Frame) {
    let area = f.area();

    let popup_width = 50u16.min(area.width.saturating_sub(4));
    let popup_height = 20u16.min(area.height.saturating_sub(4));

    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    f.render_widget(Clear, popup_area);

    let bindings = vec![
        ("q", "Quit"),
        ("Tab / Shift+Tab", "Switch screen"),
        ("j / ↓", "Move down"),
        ("k / ↑", "Move up"),
        ("Enter", "Select / Play"),
        ("Tab (in Library)", "Switch panel"),
        ("/", "Start search"),
        ("Esc", "Exit search / Close help"),
        ("Space", "Play / Pause"),
        ("n", "Next track"),
        ("p", "Previous track"),
        ("+", "Volume up"),
        ("-", "Volume down"),
        ("s", "Toggle like"),
        ("?", "Toggle help"),
    ];

    let lines: Vec<Line> = bindings
        .iter()
        .map(|(key, desc)| {
            Line::from(vec![
                Span::styled(
                    format!("{:>20}", key),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(*desc, Style::default().fg(Color::White)),
            ])
        })
        .collect();

    let help = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .title(" Keybindings (? to close) "),
    );

    f.render_widget(help, popup_area);
}
