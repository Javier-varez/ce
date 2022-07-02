use ansi_parser::AnsiParser;
use tui::{
    backend::Backend,
    style::{self, Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

fn parse_ansi_text(raw_text: &str) -> Result<Vec<Span>, std::io::Error> {
    let mut text = vec![];
    let mut parsed_escape = raw_text.ansi_parse();
    while let Some(parsed_escape) = parsed_escape.next() {
        if let ansi_parser::Output::TextBlock(new_text) = parsed_escape {
            text.push(Span::styled(new_text, style::Style::default()));
        }
    }
    Ok(text)
}

pub fn update<B: Backend>(
    terminal: &mut Terminal<B>,
    compilation: &crate::compiler_explorer::CompilationResult,
) -> Result<(), std::io::Error> {
    let mut asm_text = vec![];
    for asm in &compilation.asm {
        let fragment = parse_ansi_text(&asm.text)?;
        asm_text.push(Spans::from(fragment));
    }

    let mut stdout_text = vec![];
    for stdout in &compilation.stdout {
        let fragment = parse_ansi_text(&stdout.text)?;
        stdout_text.push(Spans::from(fragment));
    }

    let mut stderr_text = vec![];
    for stderr in &compilation.stderr {
        let fragment = parse_ansi_text(&stderr.text)?;
        stderr_text.push(Spans::from(fragment));
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
