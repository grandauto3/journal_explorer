use iced::{
    Color, Element, Length,
    alignment::Horizontal,
    widget::{
        Column, Space, button, column, container, mouse_area, pane_grid,
        pane_grid::{Axis, Configuration},
        row, scrollable,
        scrollable::{Direction, Scrollbar},
        svg, text, text_input,
    },
};
use rfd::FileDialog;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub fn main() -> iced::Result {
    iced::application("Journal Explorer", AppState::update, AppState::view).run()
}

enum AppPane {
    Input,
    Output,
    FileList,
}

struct AppState {
    input_path: PathBuf,
    file_paths: Vec<PathBuf>,
    selected_idx: Option<u32>,
    error_string: String,
    journal_output: String,
    panes: pane_grid::State<AppPane>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            input_path: PathBuf::new(),
            file_paths: vec![],
            selected_idx: None,
            error_string: String::new(),
            journal_output: String::new(),
            panes: pane_grid::State::with_configuration(Configuration::Split {
                axis: Axis::Vertical,
                ratio: 0.33,
                a: Configuration::Split {
                    axis: Axis::Horizontal,
                    ratio: 0.33,
                    a: Configuration::Pane(AppPane::Input).into(),
                    b: Configuration::Pane(AppPane::FileList).into(),
                }
                .into(),
                b: Configuration::Pane(AppPane::Output).into(),
            }),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    OnFileDialogClicked,
    OnFolderDialogClicked,
    LoadFiles,
    PathInput(String),
    FileClicked((u32, String)),
}

impl AppState {
    fn update(&mut self, message: Message) {
        match message {
            Message::FileClicked((idx, path)) => {
                self.selected_idx = Some(idx);
                self.journal_output = AppState::read_journal(Path::new(path.as_str()));
            }
            Message::PathInput(path) => self.input_path = PathBuf::from(path),
            Message::LoadFiles => {
                //reset invalid_path
                self.error_string = String::new();
                let path = &self.input_path;

                if path.is_dir() {
                    match fs::read_dir(path) {
                        Ok(content) => {
                            self.file_paths = content
                                .map(|e| match e {
                                    Ok(entry) => {
                                        if entry.path().is_file() {
                                            entry.path()
                                        } else {
                                            PathBuf::new()
                                        }
                                    }
                                    Err(e) => {
                                        self.error_string =
                                            format!("Could not read dir entry: {}", e);
                                        PathBuf::new()
                                    }
                                })
                                .filter(move |x| *x != PathBuf::new())
                                .collect::<Vec<_>>()
                        }
                        Err(e) => self.error_string = format!("Could not read dir: {}", e),
                    }
                } else if path.is_file() {
                    self.journal_output = AppState::read_journal(path);
                } else {
                    self.error_string = "Invalid Path".into();
                }
            }
            Message::OnFolderDialogClicked => {
                let folder = FileDialog::new().pick_folder();

                if let Some(path) = folder {
                    self.input_path = path;
                }
            }
            Message::OnFileDialogClicked => {
                let files = FileDialog::new()
                    .add_filter("journal files", &["journal"])
                    .pick_file();

                if let Some(path) = files {
                    self.input_path = path;
                }
            }
        }
    }

    fn read_journal(path: &Path) -> String {
        let file_content = Command::new("journalctl")
            .arg("--file")
            .arg(path.as_os_str())
            .output();
        match file_content {
            Ok(content) => {
                String::from_utf8(content.stdout).unwrap_or("Could not read stdout".into())
            }
            Err(e) => format!("Error occurred during loading {}", e),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        pane_grid(&self.panes, move |_, state, _| {
            pane_grid::Content::new(match state {
                AppPane::Input => container(
                    column![
                        row![
                            button(svg("resources/icons/folder_open.svg").width(Length::Shrink))
                                .on_press(Message::OnFolderDialogClicked),
                            Space::with_width(Length::Fixed(10f32)),
                            button(svg("resources/icons/file_open.svg").width(Length::Shrink))
                                .on_press(Message::OnFileDialogClicked),
                            Space::with_width(Length::Fixed(32f32)),
                            text_input("Enter journal path...", &self.input_path.to_string_lossy())
                                .on_input(Message::PathInput)
                        ]
                        .padding(10),
                        text(&self.error_string).color(Color::from_rgb(1f32, 0f32, 0f32)),
                        container(
                            button(text("load").center().width(Length::Fill))
                                .on_press(Message::LoadFiles)
                                .width(Length::Fill)
                        )
                        .center_x(Length::Fill)
                        .padding(10)
                    ]
                    .align_x(Horizontal::Center),
                )
                .center(Length::Fill),
                AppPane::Output => container(
                    column![
                        container(scrollable(text(&self.journal_output)).direction(
                            Direction::Both {
                                horizontal: Scrollbar::default(),
                                vertical: Scrollbar::default()
                            }
                        ))
                        .padding(10)
                        .center(Length::Fill)
                        .style(container::bordered_box)
                    ]
                    .padding(10),
                ),
                AppPane::FileList => {
                    let list_content = self
                        .file_paths
                        .iter()
                        .enumerate()
                        .map(|(idx, path)| {
                            let idx = idx as u32;
                            let file_name = path.file_name().unwrap_or_default();
                            container(mouse_area(text(file_name.to_string_lossy())).on_press(
                                Message::FileClicked((idx, path.to_string_lossy().to_string())),
                            ))
                            .style(move |_| {
                                if self.selected_idx.is_some_and(|e| e == idx) {
                                    container::Style {
                                        text_color: Some(Color::WHITE),
                                        background: Some(Color::from_rgb(0f32, 0f32, 1f32).into()),
                                        ..container::Style::default()
                                    }
                                } else {
                                    container::Style::default()
                                }
                            })
                            .into()
                        })
                        .collect::<Vec<_>>();

                    container(
                        column![
                            text(self.input_path.to_string_lossy()),
                            container(
                                container(
                                    scrollable(Column::with_children(list_content))
                                        .direction(Direction::Both {
                                            horizontal: Scrollbar::default(),
                                            vertical: Scrollbar::default(),
                                        })
                                        .height(Length::Fill),
                                )
                                .padding(5),
                            )
                            .style(container::bordered_box)
                            .height(Length::Fill)
                            .width(Length::Fill)
                        ]
                        .padding(10),
                    )
                    .center(Length::Fill)
                }
            })
        })
        .spacing(10)
        .into()
    }
}
