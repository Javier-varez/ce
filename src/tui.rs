use tui::{
    backend::Backend,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

pub fn update<B: Backend>(
    terminal: &mut Terminal<B>,
    compilation: &crate::compiler_explorer::CompilationResult,
) -> Result<(), std::io::Error> {
    terminal
        .draw(|f| {
            let parts = tui::layout::Layout::default()
                .direction(tui::layout::Direction::Vertical)
                .constraints([
                    tui::layout::Constraint::Percentage(33),
                    tui::layout::Constraint::Percentage(33),
                    tui::layout::Constraint::Percentage(33),
                ])
                .split(f.size());
            let mut text = vec![];
            for asm in &compilation.asm {
                text.push(Spans::from(format!("{}", asm.text)));
            }
            let block = Block::default().borders(Borders::ALL).title(Span::styled(
                "ASM",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ));

            let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
            f.render_widget(paragraph, parts[0]);

            let mut text = vec![];
            for stdout in &compilation.stdout {
                text.push(Spans::from(format!("{}", stdout.text)));
            }
            let block = Block::default().borders(Borders::ALL).title(Span::styled(
                "Stdout",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ));

            let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
            f.render_widget(paragraph, parts[1]);

            let mut text = vec![];
            for stderr in &compilation.stderr {
                text.push(Spans::from(format!("{}", stderr.text)));
            }
            let block = Block::default().borders(Borders::ALL).title(Span::styled(
                "Stderr",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ));

            let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
            f.render_widget(paragraph, parts[2]);
        })
        .unwrap();
    Ok(())
}
