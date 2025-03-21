use std::path::Path;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{App, InputMode};

pub fn draw_ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(f.area());

    let header = Paragraph::new("Image Converter TUI (Press 'q' to quit)")
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Header"));

    f.render_widget(header, chunks[0]);

    let items: Vec<ListItem> = app.images.iter().map(|i| {
        let path = Path::new(i);
        let file_stem = path.file_stem().unwrap().to_str().unwrap();
        let file_extension = path.extension().unwrap().to_str().unwrap();
        ListItem::new(format!("{}.{} -> {}.{}", file_stem, file_extension, file_stem, app.selected_format))
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

        InputMode::AddingImage => (
            "Enter image file path(s): ",
            Style::default().fg(Color::Green)
        ),

        InputMode::SettingOutput => (
            "Enter output folder: ",
            Style::default().fg(Color::Green)
        ),
    };

    let input = Paragraph::new(app.input_buffer.as_str())
        .style(style)
        .block(Block::default().borders(Borders::ALL).title(prompt));

    f.render_widget(input, chunks[3]);
}