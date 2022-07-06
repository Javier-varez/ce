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

#[derive(PartialEq, Debug, Clone, Copy)]
enum Widgets {
    Asm = 0,
    Stdout,
    Stderr,
}

#[derive(Clone, Copy)]
struct WidgetConfig {
    vertical_offset: u16,
    horizontal_offset: u16,
}

impl Default for WidgetConfig {
    fn default() -> Self {
        Self {
            vertical_offset: 0,
            horizontal_offset: 0,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Orientation {
    Vertical,
    Horizontal,
}

pub struct Ui {
    selected_widget: Widgets,
    widget_config: [WidgetConfig; 3],
    focus: Option<Widgets>,
    orientation: Orientation,
    data: Option<CompilationResult>,
}

impl Ui {
    pub fn new(orientation: Orientation) -> Self {
        Self {
            selected_widget: Widgets::Asm,
            widget_config: [WidgetConfig::default(); 3],
            focus: None,
            orientation,
            data: None,
        }
    }

    pub fn set_data(&mut self, compilation: CompilationResult) {
        self.data = Some(compilation);
        // Reset offsets
        self.widget_config
            .iter_mut()
            .for_each(|config| config.vertical_offset = 0);
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
                config.vertical_offset += 1;
                true
            }
            (KeyCode::Char('k'), KeyModifiers::NONE) => {
                let config = match self.selected_widget {
                    Widgets::Asm => &mut self.widget_config[0],
                    Widgets::Stdout => &mut self.widget_config[1],
                    Widgets::Stderr => &mut self.widget_config[2],
                };
                if config.vertical_offset > 0 {
                    config.vertical_offset -= 1;
                }
                true
            }
            (KeyCode::Char('l'), KeyModifiers::NONE) => {
                let config = match self.selected_widget {
                    Widgets::Asm => &mut self.widget_config[0],
                    Widgets::Stdout => &mut self.widget_config[1],
                    Widgets::Stderr => &mut self.widget_config[2],
                };
                config.horizontal_offset += 1;
                true
            }
            (KeyCode::Char('h'), KeyModifiers::NONE) => {
                let config = match self.selected_widget {
                    Widgets::Asm => &mut self.widget_config[0],
                    Widgets::Stdout => &mut self.widget_config[1],
                    Widgets::Stderr => &mut self.widget_config[2],
                };
                if config.horizontal_offset > 0 {
                    config.horizontal_offset -= 1;
                }
                true
            }
            (KeyCode::Char('J'), KeyModifiers::SHIFT)
                if self.focus.is_none() && self.orientation == Orientation::Vertical =>
            {
                self.selected_widget = match self.selected_widget {
                    Widgets::Asm => Widgets::Stdout,
                    Widgets::Stdout => Widgets::Stderr,
                    Widgets::Stderr => Widgets::Asm,
                };
                true
            }
            (KeyCode::Char('L'), KeyModifiers::SHIFT)
                if self.focus.is_none() && self.orientation == Orientation::Horizontal =>
            {
                self.selected_widget = match self.selected_widget {
                    Widgets::Asm => Widgets::Stdout,
                    Widgets::Stdout => Widgets::Stderr,
                    Widgets::Stderr => Widgets::Asm,
                };
                true
            }
            (KeyCode::Char('K'), KeyModifiers::SHIFT)
                if self.focus.is_none() && self.orientation == Orientation::Vertical =>
            {
                self.selected_widget = match self.selected_widget {
                    Widgets::Asm => Widgets::Stderr,
                    Widgets::Stdout => Widgets::Asm,
                    Widgets::Stderr => Widgets::Stdout,
                };
                true
            }
            (KeyCode::Char('H'), KeyModifiers::SHIFT)
                if self.focus.is_none() && self.orientation == Orientation::Horizontal =>
            {
                self.selected_widget = match self.selected_widget {
                    Widgets::Asm => Widgets::Stderr,
                    Widgets::Stdout => Widgets::Asm,
                    Widgets::Stderr => Widgets::Stdout,
                };
                true
            }
            (KeyCode::Enter, _) => {
                match self.focus {
                    None => {
                        self.focus = Some(self.selected_widget);
                    }
                    Some(_) => {
                        self.focus = None;
                    }
                }
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

            if let Some(execution_result) = &compilation.execution {
                for stdout in &execution_result.stdout {
                    stdout_text.extend(ansi_to_text(stdout.text.bytes()).unwrap());
                }

                for stderr in &execution_result.stderr {
                    stderr_text.extend(ansi_to_text(stderr.text.bytes()).unwrap());
                }
            }
        }

        let mut num_blocks = 0;
        let mut show_asm = false;
        if !asm_text.lines.is_empty() {
            num_blocks += 1;
            show_asm = true;
        }
        let asm_block = Self::draw_paragraph_block(
            "ASM",
            asm_text,
            self.selected_widget == Widgets::Asm,
            &self.widget_config[0],
        );
        let mut show_stdout = false;
        if !stdout_text.lines.is_empty() {
            num_blocks += 1;
            show_stdout = true;
        }
        let stdout_block = Self::draw_paragraph_block(
            "Stdout",
            stdout_text,
            self.selected_widget == Widgets::Stdout,
            &self.widget_config[1],
        );
        let mut show_stderr = false;
        if !stderr_text.lines.is_empty() {
            num_blocks += 1;
            show_stderr = true;
        }
        let stderr_block = Self::draw_paragraph_block(
            "Stderr",
            stderr_text,
            self.selected_widget == Widgets::Stderr,
            &self.widget_config[2],
        );

        if num_blocks == 0 {
            num_blocks = 1;
        }
        let percentage = 100 / num_blocks;
        let mut constraints = vec![];
        for _ in 0..num_blocks {
            constraints.push(tui::layout::Constraint::Percentage(percentage));
        }

        terminal
            .draw(|f| match self.focus {
                Some(Widgets::Asm) => {
                    f.render_widget(asm_block, f.size());
                }
                Some(Widgets::Stdout) => {
                    f.render_widget(stdout_block, f.size());
                }
                Some(Widgets::Stderr) => {
                    f.render_widget(stderr_block, f.size());
                }
                None => {
                    let parts = tui::layout::Layout::default()
                        .direction(if self.orientation == Orientation::Vertical {
                            tui::layout::Direction::Vertical
                        } else {
                            tui::layout::Direction::Horizontal
                        })
                        .constraints(constraints)
                        .split(f.size());

                    if show_asm {
                        f.render_widget(asm_block, parts[0]);
                    }
                    if show_stdout {
                        f.render_widget(stdout_block, parts[1]);
                    }
                    if show_stderr {
                        f.render_widget(stderr_block, parts[2]);
                    }
                }
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
            .scroll((config.vertical_offset, config.horizontal_offset))
    }
}
