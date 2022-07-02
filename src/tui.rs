use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::Borders;
use tui::widgets::Wrap;
use tui::widgets::{Block, Paragraph};
use tui::{backend::CrosstermBackend, Terminal};

pub fn init() -> Result<(), std::io::Error> {
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal
        .draw(|f| {
            let text = vec![Spans::from("Hello world!")];
            let block = Block::default().borders(Borders::ALL).title(Span::styled(
                "This is a block!",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ));

            let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
            f.render_widget(paragraph, f.size());
        })
        .unwrap();
    Ok(())
}

pub fn update() {}
