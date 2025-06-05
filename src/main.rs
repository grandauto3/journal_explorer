use iced::{
    Color, Element, Font, Length, Subscription, Task,
    alignment::Horizontal,
    keyboard::{Key, key::Named, on_key_press},
    padding,
    widget::{
        Column, Space, button, column, container, mouse_area, pane_grid,
        pane_grid::{Axis, Configuration},
        rich_text, row, scrollable,
        scrollable::{Direction, Scrollbar},
        span, svg, text,
        text::Span,
        text_input,
    },
};
use iced_aw::spinner;
use rfd::FileDialog;
use std::{fs, iter::once, path::PathBuf, process::Command};

pub fn main() -> iced::Result {
    iced::application("Journal Explorer", AppState::update, AppState::view)
        .subscription(AppState::subscription)
        .run()
}

enum AppPane {
    Input,
    Output,
    FileList,
}

struct AppState<'a> {
    input_path: PathBuf,
    file_paths: Vec<PathBuf>,
    selected_idx: Option<u32>,
    error_string: String,
    //this is a Vec<String> so we can fragment the output for search
    journal_output: Vec<Span<'a, Message>>,
    search_term: String,
    panes: pane_grid::State<AppPane>,
    show_spinner: bool,
    show_dir_path: bool,
}

impl Default for AppState<'_> {
    fn default() -> Self {
        Self {
            input_path: PathBuf::new(),
            file_paths: vec![],
            selected_idx: None,
            error_string: String::new(),
            journal_output: vec![],
            search_term: String::new(),
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
            show_spinner: false,
            show_dir_path: false,
        }
    }
}

#[derive(Debug, Clone)]
enum KeyBoardDirection {
    Up,
    Down,
}

#[derive(Debug, Clone)]
enum Message {
    OnFileDialogClicked,
    OnFolderDialogClicked,
    OnLoadClicked,
    OnFileLoaded(String),
    PathInput(String),
    FileClicked((u32, String)),
    Search(String),
    OnArrowKeyPressed(KeyBoardDirection),
}

impl AppState<'_> {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FileClicked((idx, path)) => {
                self.selected_idx = Some(idx);
                self.show_spinner = true;
                Task::perform(
                    AppState::read_journal(PathBuf::from(path)),
                    Message::OnFileLoaded,
                )
            }
            Message::PathInput(path) => {
                self.input_path = PathBuf::from(path);
                Task::none()
            }
            Message::OnLoadClicked => {
                //reset invalid_path
                self.error_string = String::new();
                let path = &self.input_path;

                if path.is_dir() {
                    match Self::load_dir_content(path) {
                        Ok(paths) => {
                            self.file_paths = paths;
                        }
                        Err(e) => self.error_string = format!("Error when reading paths: {}", e),
                    };

                    Task::none()
                } else if path.is_file() {
                    self.show_spinner = true;
                    Task::perform(
                        AppState::read_journal(PathBuf::from(path)),
                        Message::OnFileLoaded,
                    )
                } else {
                    self.error_string = "Invalid Path".into();
                    Task::none()
                }
            }
            Message::OnFolderDialogClicked => {
                let folder = FileDialog::new().pick_folder();

                if let Some(path) = folder {
                    self.input_path = path;
                }

                match Self::load_dir_content(&self.input_path) {
                    Ok(paths) => {
                        self.file_paths = paths;
                    }
                    Err(e) => self.error_string = format!("Error when reading paths: {}", e),
                };

                Task::none()
            }
            Message::OnFileDialogClicked => {
                let files = FileDialog::new()
                    .add_filter("journal files", &["journal"])
                    .pick_file();

                if let Some(path) = files {
                    self.input_path = path;
                }
                Task::perform(
                    AppState::read_journal(self.input_path.clone()),
                    Message::OnFileLoaded,
                )
            }
            Message::OnFileLoaded(output) => {
                self.show_spinner = false;
                self.journal_output = vec![span(output)];
                Task::none()
            }
            Message::Search(search_term) => {
                self.search_term = search_term;
                let joined = self
                    .journal_output
                    .iter()
                    .map(|e| e.text.to_string())
                    .collect::<String>();
                let fragmented_output =
                    split_by_delimiter(joined.as_str(), self.search_term.as_str())
                        .iter()
                        .map(|e| span(e.to_string()))
                        .collect::<Vec<Span<'_, Message, Font>>>();

                self.journal_output = fragmented_output
                    .into_iter()
                    .map(|e| {
                        if e.text == self.search_term {
                            e.background(Color::from_rgb(1f32, 1f32, 0f32))
                        } else {
                            e
                        }
                    })
                    .collect::<Vec<_>>();

                Task::none()
            }
            Message::OnArrowKeyPressed(dir) => {
                if let Some(idx) = &mut self.selected_idx {
                    match dir {
                        KeyBoardDirection::Down => {
                            let upper_limit = self.file_paths.len();
                            if (*idx as usize) < upper_limit - 1 {
                                *idx += 1
                            }
                        }
                        KeyBoardDirection::Up => *idx = idx.saturating_sub(1),
                    }
                }
                self.show_spinner = true;
                Task::perform(
                    AppState::read_journal(
                        self.file_paths
                            .get(self.selected_idx.unwrap() as usize)
                            .unwrap()
                            .clone(),
                    ),
                    Message::OnFileLoaded,
                )
            }
        }
    }

    fn load_dir_content(path: &PathBuf) -> anyhow::Result<Vec<PathBuf>> {
        fs::read_dir(path)?
            .map(|e| {
                let entry = e?;

                if entry.path().is_file() {
                    Ok(entry.path())
                } else {
                    Ok(PathBuf::new())
                }
            })
            .filter(|x| x.as_ref().is_ok_and(|e| *e != PathBuf::new()))
            .collect::<anyhow::Result<Vec<_>>>()
    }

    async fn read_journal(path: PathBuf) -> String {
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
                                .on_press(Message::OnLoadClicked)
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
                        text_input("Search...", &self.search_term).on_input(Message::Search),
                        container(
                            scrollable(
                                container(rich_text(&self.journal_output))
                                    .padding(padding::bottom(10))
                            )
                            .direction(Direction::Both {
                                horizontal: Scrollbar::default(),
                                vertical: Scrollbar::default()
                            })
                        )
                        .padding(10)
                        .center(Length::Fill)
                        .style(container::bordered_box)
                    ]
                    .spacing(5)
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
                            row![text(if self.show_dir_path {
                                self.input_path.to_string_lossy()
                            } else {
                                "".into()
                            }),]
                            .push_maybe(if self.show_spinner {
                                Some(spinner::Spinner::new())
                            } else {
                                None
                            }),
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
                        .padding(10)
                        .width(Length::Fill),
                    )
                    .center(Length::Fill)
                }
            })
        })
        .spacing(10)
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        on_key_press(|key, _| {
            if let Key::Named(keycode) = key {
                if keycode == Named::ArrowDown {
                    Some(Message::OnArrowKeyPressed(KeyBoardDirection::Down))
                } else if keycode == Named::ArrowUp {
                    Some(Message::OnArrowKeyPressed(KeyBoardDirection::Up))
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
}

pub fn split_by_delimiter<'a>(text: &'a str, delimiter: &'a str) -> Vec<&'a str> {
    text.split_terminator(delimiter)
        .flat_map(|part| {
            once(part).chain(once(delimiter)).take({
                let last_element = text.split(delimiter).last().unwrap();
                if part == last_element { 1 } else { 2 }
            })
        })
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use crate::split_by_delimiter;

    #[test]
    fn get_separated_string() {
        let result = split_by_delimiter("HelloRustWorld", "Rust");
        assert_eq!(result, vec!["Hello", "Rust", "World"]);
    }

    #[test]
    fn get_separated_string_by_matching_long() {
        let test_string = "HelloRustWorldKek";

        let result = split_by_delimiter(test_string, "Rust");
        assert_eq!(result, vec!["Hello", "Rust", "WorldKek"]);
    }
    #[test]
    fn get_separated_string_by_matching_long_with_whitespace() {
        let test_string = "Hello Rust World Kek";

        let result = split_by_delimiter(test_string, "Rust");
        assert_eq!(result, vec!["Hello ", "Rust", " World Kek"]);
    }
    #[test]
    fn get_separated_string_by_matching_multiple() {
        let test_string = "HelloRustWorldKekRust";

        let result = split_by_delimiter(test_string, "Rust");
        assert_eq!(result, vec!["Hello", "Rust", "WorldKek", "Rust"]);
    }
}
