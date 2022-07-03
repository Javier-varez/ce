use ansi_to_tui::ansi_to_text;
use tui::{
    backend::Backend,
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

pub fn update<B: Backend>(
    terminal: &mut Terminal<B>,
    compilation: &crate::compiler_explorer::CompilationResult,
) -> Result<(), std::io::Error> {
    let mut asm_text = Text::default();
    for asm in &compilation.asm {
        asm_text.extend(ansi_to_text(asm.text.bytes()).unwrap());
    }

    let mut stdout_text = Text::default();
    for stdout in &compilation.stdout {
        stdout_text.extend(ansi_to_text(stdout.text.bytes()).unwrap());
    }

    let mut stderr_text = Text::default();
    for stderr in &compilation.stderr {
        stderr_text.extend(ansi_to_text(stderr.text.bytes()).unwrap());
    }

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

            let paragraph = Paragraph::new(asm_text)
                .block(block)
                .wrap(Wrap { trim: false });
            f.render_widget(paragraph, parts[0]);

            let block = Block::default().borders(Borders::ALL).title(Span::styled(
                "Stdout",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ));

            let paragraph = Paragraph::new(stdout_text)
                .block(block)
                .wrap(Wrap { trim: false });
            f.render_widget(paragraph, parts[1]);

            let block = Block::default().borders(Borders::ALL).title(Span::styled(
                "Stderr",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ));

            let paragraph = Paragraph::new(stderr_text)
                .block(block)
                .wrap(Wrap { trim: false });
            f.render_widget(paragraph, parts[2]);
        })
        .unwrap();
    Ok(())
}
