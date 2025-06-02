use iced::Element;
use iced::widget::PaneGrid;
use iced::widget::{Row, button, column, container, pane_grid, row, svg, text, text_input};
use iced::{Border, Color, border};
use rfd::FileDialog;

pub fn main() -> iced::Result {
    iced::application("Journal Explorer", AppState::update, AppState::view).run()
}

enum AppPane {
    InputPane,
    OutputPane,
}

struct AppState {
    path: String,
    journal_output: String,
    pane: pane_grid::State<AppPane>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            path: String::new(),
            journal_output: String::new(),
            pane: pane_grid::State::new(AppPane::InputPane).0,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    OnFileDialogClicked,
    ExecuteJournal,
    PathInput(String),
}

impl AppState {
    fn update(&mut self, message: Message) {
        match message {
            Message::PathInput(path) => self.path = path,
            Message::ExecuteJournal => println!("To run journalctl --file {}", &self.path),
            Message::OnFileDialogClicked => {
                let files = FileDialog::new().pick_folder();

                if let Some(path) = files {
                    self.path = path.into_os_string().into_string().unwrap();
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        pane_grid(&self.pane, move |pane, state, is_maximized| {
            pane_grid::Content::new(match state {
                AppPane::InputPane => column![
                    row![
                        button(svg("resources/icons/folder_open.svg"))
                            .on_press(Message::OnFileDialogClicked),
                        button(svg("resources/icons/file_open.svg"))
                            .on_press(Message::OnFileDialogClicked),
                        text_input("Enter journal path...", &self.path)
                            .on_input(Message::PathInput)
                    ],
                    button("run").on_press(Message::ExecuteJournal)
                ],
                AppPane::OutputPane => {
                    column![container(text(&self.journal_output)).style(container::bordered_box)]
                }
            })
        })
        .into()
    }
}
