/*
    Feature ideas:
    - fix side left side panel
    - search jumps to every occurrence

    Bugs:
    - closing FileDialog logs an error
    - file_list does not move on key input
*/
mod utils;

use iced::{
    Color, Element, Font, Length, Padding, Subscription, Task,
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
use iced_aw::helpers::selection_list_with;
use iced_aw::{selection_list, spinner, style};
use rfd::FileDialog;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

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

#[repr(transparent)]
struct PathWrapper(PathBuf);

impl Display for PathWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0.clone().into_os_string().into_string().unwrap()
        )
    }
}

struct AppState<'a> {
    input_path: PathBuf,
    //needs to be string for selection_list, PathBuf does not implement Display
    file_paths: Vec<String>,
    selected_idx: Option<usize>,
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
    FileClicked(usize, String),
    Search(String),
    OnArrowKeyPressed(KeyBoardDirection),
}

impl AppState<'_> {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FileClicked(idx, path) => {
                self.selected_idx = Some(idx);
                self.show_spinner = true;
                Task::perform(
                    utils::read_journal(PathBuf::from(path)),
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
                    match utils::load_dir_content(path) {
                        Ok(paths) => {
                            self.file_paths = utils::path_vec_to_string_vec(&paths);
                        }
                        Err(e) => self.error_string = format!("Error when reading paths: {}", e),
                    };

                    Task::none()
                } else if path.is_file() {
                    self.show_spinner = true;
                    Task::perform(
                        utils::read_journal(PathBuf::from(path)),
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

                match utils::load_dir_content(&self.input_path) {
                    Ok(paths) => {
                        self.file_paths = utils::path_vec_to_string_vec(&paths);
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
                    utils::read_journal(self.input_path.clone()),
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
                    utils::split_by_delimiter(joined.as_str(), self.search_term.as_str())
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
                    utils::read_journal(PathBuf::from(
                        self.file_paths
                            .get(self.selected_idx.unwrap())
                            .unwrap()
                            .clone(),
                    )),
                    Message::OnFileLoaded,
                )
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
                AppPane::FileList => container(
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
                        container(selection_list_with(
                            { self.file_paths.as_slice() },
                            Message::FileClicked,
                            16f32,
                            Padding::from(5f32),
                            |t, s| {
                                let box_style = container::bordered_box(t);
                                let primary = style::selection_list::primary(t, s);
                                let default = style::selection_list::Style {
                                    background: box_style.background.unwrap(),
                                    border_color: box_style.border.color,
                                    border_width: box_style.border.width,

                                    ..primary
                                };

                                match s {
                                    style::status::Status::Selected => primary,
                                    _ => default,
                                }
                            },
                            None,
                            Font::default()
                        ))
                        .style(container::bordered_box)
                        .height(Length::Fill)
                        .width(Length::Fill)
                    ]
                    .padding(10)
                    .width(Length::Fill),
                )
                .center(Length::Fill),
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
