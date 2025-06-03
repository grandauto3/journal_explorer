use iced::widget::scrollable::{Direction, Scrollbar};
use iced::{
    Color, Element, Length,
    alignment::Horizontal,
    widget::{
        Space, button, column, container, pane_grid,
        pane_grid::{Axis, Configuration},
        row, scrollable, svg, text, text_input,
    },
};
use rfd::FileDialog;
use std::{path::PathBuf, process::Command};

pub fn main() -> iced::Result {
    iced::application("Journal Explorer", AppState::update, AppState::view).run()
}

enum AppPane {
    Input,
    Output,
    FileList,
}

struct AppState {
    path: PathBuf,
    error_string: String,
    journal_output: String,
    panes: pane_grid::State<AppPane>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
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
}

impl AppState {
    fn update(&mut self, message: Message) {
        match message {
            Message::PathInput(path) => self.path = PathBuf::from(path),
            Message::LoadFiles => {
                //reset invalid_path
                self.error_string = String::new();
                let path = &self.path;

                if path.is_dir() {
                } else if path.is_file() {
                    let file_content = Command::new("journalctl")
                        .arg("--file")
                        .arg(path.as_os_str())
                        .output();
                    self.journal_output = match file_content {
                        Ok(content) => String::from_utf8(content.stdout)
                            .unwrap_or("Could not read stdout".into()),
                        Err(e) => format!("Error occurred during loading {}", e),
                    };
                } else {
                    self.error_string = "Invalid Path".into();
                }
            }
            Message::OnFolderDialogClicked => {
                let folder = FileDialog::new().pick_folder();

                if let Some(path) = folder {
                    self.path = path;
                }
            }
            Message::OnFileDialogClicked => {
                let files = FileDialog::new()
                    .add_filter("journal files", &["journal"])
                    .pick_file();

                if let Some(path) = files {
                    self.path = path;
                }
            }
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
                            text_input("Enter journal path...", &self.path.to_string_lossy())
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
                AppPane::FileList => container(
                    column![
                        container(scrollable(text("files")))
                            .center(Length::Fill)
                            .style(container::bordered_box)
                    ]
                    .padding(10),
                ),
            })
        })
        .spacing(10)
        .into()
    }
}
