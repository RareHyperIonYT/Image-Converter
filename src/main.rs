use std::{
    error::Error,
    io,
    path::{Path, PathBuf},
    time::Duration,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use image::{DynamicImage, ImageOutputFormat};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

struct App {
    images: Vec<String>,
    output_folder: String,
    selected_format: String,
    formats: Vec<&'static str>,
    input_mode: InputMode,
    input_buffer: String,
}

enum InputMode {
    Normal,
    AddingImage,
    SettingOutput,
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

fn parse_image_paths(input: &str) -> Vec<String> {
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

fn convert_image(input: &str, output_folder: &str, format: &str) -> Result<(), Box<dyn Error>> {
    let img: DynamicImage = image::open(input)?;
    let input_path = Path::new(input);
    let filename = input_path
        .file_stem()
        .ok_or("Invalid file name")?
        .to_string_lossy();

    let output_path = PathBuf::from(output_folder);
    std::fs::create_dir_all(&output_path)?;

    let output_path = output_path.join(format!("{}.{}", filename, format));

    let output_format = match format {
        "jpeg" | "jpg" => ImageOutputFormat::Jpeg(80),
        "webp" => ImageOutputFormat::WebP,
        "gif" => ImageOutputFormat::Gif,
        _ => ImageOutputFormat::Png,
    };

    img.write_to(&mut std::fs::File::create(&output_path)?, output_format)?;
    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                ])
                .split(f.size());

            let header = Paragraph::new("Image Converter TUI (Press 'q' to quit)")
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Header"));
            f.render_widget(header, chunks[0]);

            let items: Vec<ListItem> = app.images.iter().map(|i| {
                let file_stem = Path::new(i)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy();
                ListItem::new(format!("{} -> {}.{}", i, file_stem, app.selected_format))
            }).collect();

            let images_list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Images"));
            f.render_widget(images_list, chunks[1]);

            let status_text = format!(
                "Output Folder: {}\nFormat: {}",
                app.output_folder, app.selected_format
            );
            let status = Paragraph::new(status_text)
                .block(Block::default().borders(Borders::ALL).title("Settings"));
            f.render_widget(status, chunks[2]);

            let (prompt, style) = match app.input_mode {
                InputMode::Normal => (
                    "Press 'a' to add image, 'o' to set output folder, 'f' to change format, 'c' to convert",
                    Style::default(),
                ),
                InputMode::AddingImage => ("Enter image file path(s): ", Style::default().fg(Color::Green)),
                InputMode::SettingOutput => ("Enter output folder: ", Style::default().fg(Color::Green)),
            };
            let input = Paragraph::new(app.input_buffer.as_ref())
                .style(style)
                .block(Block::default().borders(Borders::ALL).title(prompt));
            f.render_widget(input, chunks[3]);
        })?;

        if !event::poll(Duration::from_millis(200))? {
            continue;
        }

        match event::read()? {
            Event::Key(key) => match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('a') => {
                        app.images.clear();
                        app.input_mode = InputMode::AddingImage;
                        app.input_buffer.clear();
                    }
                    KeyCode::Char('o') => {
                        app.input_mode = InputMode::SettingOutput;
                        app.input_buffer.clear();
                    }
                    KeyCode::Char('f') | KeyCode::Char('F') => {
                        if let Some(idx) = app.formats.iter().position(|&f| f == app.selected_format) {
                            app.selected_format = app.formats[(idx + 1) % app.formats.len()].to_string();
                        }
                    }
                    KeyCode::Char('c') => {
                        for path in &mut app.images {
                            let original = path.clone();
                            *path = match convert_image(&original, &app.output_folder, &app.selected_format) {
                                Ok(_) => format!("Converted {} successfully", original),
                                Err(e) => format!("Error converting {}: {}", original, e),
                            }
                        }
                    }
                    _ => {}
                },
                InputMode::AddingImage | InputMode::SettingOutput => match key.code {
                    KeyCode::Enter => {
                        if !app.input_buffer.is_empty() {
                            match app.input_mode {
                                InputMode::AddingImage => {
                                    app.images.extend(parse_image_paths(&app.input_buffer));
                                }
                                InputMode::SettingOutput => {
                                    app.output_folder = app.input_buffer.trim().to_string();
                                }
                                _ => {}
                            }
                        }
                        app.input_mode = InputMode::Normal;
                        app.input_buffer.clear();
                    }
                    KeyCode::Char(c) => app.input_buffer.push(c),
                    KeyCode::Backspace => {
                        app.input_buffer.pop();
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                        app.input_buffer.clear();
                    }
                    _ => {}
                },
            },
            _ => {}
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::default();
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}