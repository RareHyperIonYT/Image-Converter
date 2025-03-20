use crossterm::event::{KeyCode, KeyEvent};
use std::error::Error;

pub enum InputMode {
    Normal,
    AddingImage,
    SettingOutput,
}

pub struct App {
    pub images: Vec<String>,
    pub output_folder: String,
    pub selected_format: String,
    pub formats: Vec<&'static str>,
    pub input_mode: InputMode,
    pub input_buffer: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            images: Vec::new(),
            output_folder: "./output".into(),
            selected_format: "png".into(),
            formats: vec!["png", "jpeg", "webp", "gif"],
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
        }
    }
}

impl App {
    pub fn handle_key_event<F>(&mut self, key: KeyEvent, convert_fn: F) -> bool where F: Fn(&str, &str, &str) -> Result<(), Box<dyn Error>> {
        match self.input_mode {
            InputMode::Normal => match key.code {
                KeyCode::Char('q') => return true,

                KeyCode::Char('a') => {
                    self.images.clear();
                    self.input_mode = InputMode::AddingImage;
                    self.input_buffer.clear();
                }

                KeyCode::Char('o') => {
                    self.input_mode = InputMode::SettingOutput;
                    self.input_buffer.clear();
                }

                KeyCode::Char('f') | KeyCode::Char('F') => {
                    if let Some(idx) = self.formats.iter().position(|&f| f == self.selected_format) {
                        self.selected_format = self.formats[(idx + 1) % self.formats.len()].to_string();
                    }
                }

                KeyCode::Char('c') => {
                    for path in &mut self.images {
                        let original = path.clone();
                        *path = match convert_fn(&original, &self.output_folder, &self.selected_format) {
                            Ok(_) => format!("Converted {} successfully", original),
                            Err(e) => format!("Error converting {}: {}", original, e),
                        }
                    }
                }

                _ => {}
            },
            InputMode::AddingImage | InputMode::SettingOutput => match key.code {
                KeyCode::Enter => {
                    if !self.input_buffer.is_empty() {
                        match self.input_mode {
                            InputMode::AddingImage => {
                                self.images.extend(Self::parse_image_paths(&self.input_buffer));
                            }

                            InputMode::SettingOutput => {
                                self.output_folder = self.input_buffer.trim().to_string();
                            }

                            _ => {}
                        }
                    }

                    self.input_mode = InputMode::Normal;
                    self.input_buffer.clear();
                }

                KeyCode::Char(c) => self.input_buffer.push(c),

                KeyCode::Backspace => {
                    self.input_buffer.pop();
                }

                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.input_buffer.clear();
                }

                _ => {}
            },
        }

        false
    }

    pub fn parse_image_paths(input: &str) -> Vec<String> {
        let mut paths = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;

        for c in input.chars() {
            match c {
                '"' => in_quotes = !in_quotes,
                c if (c == ' ' || c == ',' || c == '\n') && !in_quotes => {
                    if !current.is_empty() {
                        paths.push(std::mem::take(&mut current));
                    }
                }
                _ => current.push(c),
            }
        }

        if !current.is_empty() {
            paths.push(current);
        }

        paths
    }
}