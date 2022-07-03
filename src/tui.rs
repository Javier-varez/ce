use crate::compiler_explorer::CompilationResult;

use ansi_to_tui::ansi_to_text;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::{
    backend::Backend,
    style::{Color, Modifier, Style},
    text::{Span, Text},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Terminal,
};

#[derive(PartialEq, Debug)]
enum Widgets {
    Asm = 0,
    Stdout,
    Stderr,
}

#[derive(Clone, Copy)]
struct WidgetConfig {
    offset: u16,
}

impl Default for WidgetConfig {
    fn default() -> Self {
        Self { offset: 0 }
    }
}

pub struct Ui {
    selected_widget: Widgets,
    widget_config: [WidgetConfig; 3],
    data: Option<CompilationResult>,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            selected_widget: Widgets::Asm,
            widget_config: [WidgetConfig::default(); 3],
            data: None,
        }
    }

    pub fn set_data(&mut self, compilation: CompilationResult) {
        self.data = Some(compilation);
    }

    pub fn handle_key_event<B: Backend>(
        &mut self,
        event: KeyEvent,
        terminal: &mut Terminal<B>,
    ) -> Result<(), std::io::Error> {
        let KeyEvent { code, modifiers } = event;
        let update_ui = match (code, modifiers) {
            (KeyCode::Char('j'), KeyModifiers::NONE) => {
                let config = match self.selected_widget {
                    Widgets::Asm => &mut self.widget_config[0],
                    Widgets::Stdout => &mut self.widget_config[1],
                    Widgets::Stderr => &mut self.widget_config[2],
                };
                config.offset += 1;
                true
            }
            (KeyCode::Char('k'), KeyModifiers::NONE) => {
                let config = match self.selected_widget {
                    Widgets::Asm => &mut self.widget_config[0],
                    Widgets::Stdout => &mut self.widget_config[1],
                    Widgets::Stderr => &mut self.widget_config[2],
                };
                if config.offset > 0 {
                    config.offset -= 1;
                }
                true
            }
            (KeyCode::Char('J'), KeyModifiers::SHIFT) => {
                self.selected_widget = match self.selected_widget {
                    Widgets::Asm => Widgets::Stdout,
                    Widgets::Stdout => Widgets::Stderr,
                    Widgets::Stderr => Widgets::Asm,
                };
                true
            }
            (KeyCode::Char('K'), KeyModifiers::SHIFT) => {
                self.selected_widget = match self.selected_widget {
                    Widgets::Asm => Widgets::Stderr,
                    Widgets::Stdout => Widgets::Asm,
                    Widgets::Stderr => Widgets::Stdout,
                };
                true
            }
            _ => false,
        };
        if update_ui {
            self.draw(terminal)?;
        }
        Ok(())
    }

    pub fn draw<B: Backend>(&self, terminal: &mut Terminal<B>) -> Result<(), std::io::Error> {
        let mut asm_text = Text::default();
        let mut stdout_text = Text::default();
        let mut stderr_text = Text::default();

        if let Some(compilation) = &self.data {
            for asm in &compilation.asm {
                asm_text.extend(ansi_to_text(asm.text.bytes()).unwrap());
            }

            for stdout in &compilation.stdout {
                stdout_text.extend(ansi_to_text(stdout.text.bytes()).unwrap());
            }

            for stderr in &compilation.stderr {
                stderr_text.extend(ansi_to_text(stderr.text.bytes()).unwrap());
            }
        }

        let asm_block = Self::draw_paragraph_block(
            "ASM",
            asm_text,
            self.selected_widget == Widgets::Asm,
            &self.widget_config[0],
        );
        let stdout_block = Self::draw_paragraph_block(
            "Stdout",
            stdout_text,
            self.selected_widget == Widgets::Stdout,
            &self.widget_config[1],
        );
        let stderr_block = Self::draw_paragraph_block(
            "Stderr",
            stderr_text,
            self.selected_widget == Widgets::Stderr,
            &self.widget_config[2],
        );

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

                f.render_widget(asm_block, parts[0]);
                f.render_widget(stdout_block, parts[1]);
                f.render_widget(stderr_block, parts[2]);
            })
            .unwrap();
        Ok(())
    }

    fn draw_paragraph_block<'a>(
        title: &'a str,
        text: Text<'a>,
        selected: bool,
        config: &WidgetConfig,
    ) -> Paragraph<'a> {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                title,
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ))
            .border_type(if selected {
                BorderType::Thick
            } else {
                BorderType::Plain
            });

        Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((config.offset, 0))
    }
}
