use iced::Task;
use iced::widget::svg;
use iced::widget::{Row, Svg, button, column, row, text, text_input};
use rfd::FileDialog;

pub fn main() -> iced::Result {
    iced::application("Counter", AppState::update, AppState::view).run()
}

#[derive(Default)]
struct AppState {
    path: String,
    journal_output: String,
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

    fn view(&self) -> Row<Message> {
        row![
            column![
                row![
                    button(svg("resources/icons/folder_open.svg")).on_press(Message::OnFileDialogClicked),
                    button(svg("resources/icons/file_open.svg")).on_press(Message::OnFileDialogClicked),
                    text_input("Enter journal path...", &self.path).on_input(Message::PathInput)
                ],
                button("run").on_press(Message::ExecuteJournal)
            ],
            column![text(&self.journal_output)]
        ]
    }
}
