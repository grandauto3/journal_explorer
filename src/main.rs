use iced::{
    Element,
    widget::{
        button, column, container, pane_grid,
        pane_grid::{Axis, Configuration},
        responsive, row, svg, text, text_input,
    },
};
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
    panes: pane_grid::State<AppPane>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            path: String::new(),
            journal_output: String::new(),
            panes: pane_grid::State::with_configuration(Configuration::Split {
                axis: Axis::Vertical,
                ratio: 0.5,
                a: Configuration::Pane(AppPane::InputPane).into(),
                b: Configuration::Pane(AppPane::OutputPane).into(),
            }), // pane_grid::State::new(AppPane::InputPane).0,
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
        pane_grid(&self.panes, move |pane, state, is_maximized| {
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
                    column![
                        container(responsive(move |e| text(format!(
                            "hello {}",
                            &self.journal_output
                        ))
                        .into()))
                        .style(container::bordered_box)
                    ]
                }
            })
        })
        .into()
    }
}
