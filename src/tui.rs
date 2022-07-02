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
            f.render_widget(paragraph, f.size());
        })
        .unwrap();
    Ok(())
}
