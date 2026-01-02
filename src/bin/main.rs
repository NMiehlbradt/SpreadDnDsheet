use dnd_spreadsheet::language::ast::{pretty_print_result, AST};
use dnd_spreadsheet::reactive::sheet::{CellId, Sheet};
use iced::alignment::Vertical;
use iced::widget::{button, column, container, scrollable, text, text_editor};
use iced::Length::Fill;
use iced::Element;

pub fn main() -> iced::Result {
    iced::run(State::update, State::view)
}

struct State {
    sheet: Sheet<AST>,
    cells: Vec<CellId>,
    editor_contents: text_editor::Content,
    selected_cell: Option<CellId>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            sheet: Sheet::new(),
            cells: vec![],
            editor_contents: text_editor::Content::new(),
            selected_cell: None,
        }
    }
}

impl State {
    fn update(&mut self, message: Message) {
        match message {
            Message::NewCell => {
                let cell_name = format!("A{}", self.cells.len() + 1);
                self.cells
                    .push(self.sheet.add_cell(cell_name, "0").unwrap());
            }
            Message::Edit(action) => {
                self.editor_contents.perform(action);
            }
            Message::SelectCell(id) => {
                let text = self.sheet.get_cell_text(&id).unwrap();
                self.editor_contents = text_editor::Content::with_text(text);
                self.selected_cell = Some(id);
            }
            Message::UpdateCell => {
                if let Some(id) = &self.selected_cell {
                    self.sheet.update_cell(id, self.editor_contents.text());
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        container(column![
            button("Add cell").on_press(Message::NewCell).width(Fill),
            scrollable(self.draw_cells()).height(Fill).width(Fill),
            container(column![
                text_editor(&self.editor_contents).on_action(Message::Edit),
                button("Update").on_press(Message::UpdateCell).width(Fill),
            ])
            .align_y(Vertical::Bottom),
        ])
        .into()
    }

    fn draw_cells(&self) -> impl Into<Element<'_, Message>> {
        column(self.cells.iter().map(|id| {
            let cell_name = self.sheet.get_cell_name(id);
            let cell_value = self.sheet.get_cell_value(id).unwrap();
            let button =button(column![text(cell_name), text(format!("{}", pretty_print_result(cell_value))),])
                .on_press(Message::SelectCell(id.clone()));
            if self.selected_cell.as_ref().map_or(false, |selected| selected == id) {
                button.style(button::success)
            } else {
                button
            }.into()
        }))
    }
}

#[derive(Debug, Clone)]
enum Message {
    NewCell,
    Edit(text_editor::Action),
    SelectCell(CellId),
    UpdateCell,
}
