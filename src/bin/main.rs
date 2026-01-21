use dnd_spreadsheet::language::ast::{AST, EvaluatedValue, Function, Value};
use dnd_spreadsheet::language::bultins::stringify_builtin;
use dnd_spreadsheet::language::s_exprs::ToSExpr;
use dnd_spreadsheet::language::validate_name;
use dnd_spreadsheet::reactive::sheet::{CellId, Sheet};
use iced::Length;
use iced::alignment::Vertical;
use iced::widget::container::Style;
use iced::widget::{
    button, column, container, row, scrollable, stack, text, text_editor,
};
use iced::{Color, Element, Theme};

pub fn main() -> iced::Result {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }

    iced::run(State::update, State::view)
}

struct State {
    sheet: Sheet<AST>,
    cells: Vec<CellId>,
    editor_contents: text_editor::Content,
    selected_cell: Option<CellId>,
    sub_state: SubState,
}

enum SubState {
    None,
    NewCell {
        editor_contents: text_editor::Content,
    },
}

impl Default for State {
    fn default() -> Self {
        Self {
            sheet: Sheet::new(),
            cells: vec![],
            editor_contents: text_editor::Content::new(),
            selected_cell: None,
            sub_state: SubState::None,
        }
    }
}

impl State {
    fn update(&mut self, message: Message) {
        match (&mut self.sub_state, message) {
            (SubState::None, Message::MainEditorMessage(msg)) => match msg {
                MainEditorMessage::NewCell => {
                    self.sub_state = SubState::NewCell {
                        editor_contents: text_editor::Content::new(),
                    };
                }
                MainEditorMessage::Edit(action) => {
                    self.editor_contents.perform(action);
                }
                MainEditorMessage::SelectCell(id) => {
                    let text = self.sheet.get_cell_text(&id).unwrap();
                    self.editor_contents = text_editor::Content::with_text(text);
                    self.selected_cell = Some(id);
                }
                MainEditorMessage::UpdateCell => {
                    if let Some(id) = &self.selected_cell {
                        self.sheet.update_cell(id, self.editor_contents.text());
                    }
                }
            },
            (SubState::NewCell { editor_contents }, Message::NewCellEditorMessage(msg)) => {
                match msg {
                    NewCellEditorMessage::Edit(action) => editor_contents.perform(action),
                    NewCellEditorMessage::Submit => {
                        if validate_name(&editor_contents.text())
                            && let Some(new_id) = self.sheet.add_cell(editor_contents.text(), "0")
                        {
                            self.cells.push(new_id);
                            self.sub_state = SubState::None;
                        }
                    }
                    NewCellEditorMessage::Cancel => {
                        self.sub_state = SubState::None;
                    }
                }
            }
            _ => {}
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let base = self.base();
        match &self.sub_state {
            SubState::None => base,
            SubState::NewCell { editor_contents } => {
                stack![base, State::draw_new_file_dialogue(editor_contents),].into()
            }
        }
    }

    fn base(&self) -> Element<'_, Message> {
        container(column![
            button("Add cell")
                .on_press(Message::MainEditorMessage(MainEditorMessage::NewCell))
                .width(Length::Fill),
            scrollable(self.draw_cells())
                .height(Length::Fill)
                .width(Length::Fill),
            container(column![
                text_editor(&self.editor_contents).on_action(|action| Message::MainEditorMessage(
                    MainEditorMessage::Edit(action)
                )),
                button("Update")
                    .on_press(Message::MainEditorMessage(MainEditorMessage::UpdateCell))
                    .width(Length::Fill),
            ])
            .align_y(Vertical::Bottom),
        ])
        .into()
    }

    fn draw_cells(&self) -> impl Into<Element<'_, Message>> {
        column(self.cells.iter().map(|id| {
            let cell_name = self.sheet.get_cell_name(id);
            let cell_value = self.sheet.get_cell_value(id).unwrap();
            let button = button(column![
                text(cell_name),
                match cell_value {
                    Ok(v) => Self::draw_value(v),
                    Err(e) => text(format!("Error: {}", e.message)).into(),
                },
            ])
            .on_press(Message::MainEditorMessage(MainEditorMessage::SelectCell(
                id.clone(),
            )));
            if self
                .selected_cell
                .as_ref()
                .map_or(false, |selected| selected == id)
            {
                button.style(button::success)
            } else {
                button
            }
            .into()
        }))
    }

    fn draw_value(value: &EvaluatedValue) -> Element<'_, Message> {
        match &value.0 {
            Value::Unit => text(format!("()")).into(),
            Value::Integer(i) => text(format!("{}", i)).into(),
            Value::String(s) => text(s).into(),
            Value::Boolean(b) => text(format!("{}", b)).into(),
            Value::Record(btree_map) => {
                column(btree_map.iter().map(|(k, v)| {
                    row![text(format!("{}: ", k)), Self::draw_value(v)].spacing(20).into()
                })).into()
            }
            Value::List(items) => column(items.iter().map(|item| Self::draw_value(item))).into(),
            Value::Function(function) => match function {
                Function::Lambda(args, body) => {
                    text(format!("fn ({}) -> {}", args.join(", "), body.to_s_expr())).into()
                }
                Function::Builtin(name) => {
                    text(format!("builtin {}", stringify_builtin(*name))).into()
                }
            },
        }
    }

    fn draw_new_file_dialogue(editor: &text_editor::Content) -> Element<'_, Message> {
        container(column![
            container("New Cell").center_x(Length::Fill),
            text_editor(editor).on_action(|action| Message::NewCellEditorMessage(
                NewCellEditorMessage::Edit(action)
            )),
            row![
                button(
                    container("Submit")
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                )
                .on_press(Message::NewCellEditorMessage(NewCellEditorMessage::Submit))
                .width(Length::Fill)
                .height(Length::Shrink),
                button(
                    container("Cancel")
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                )
                .on_press(Message::NewCellEditorMessage(NewCellEditorMessage::Cancel))
                .width(Length::Fill)
                .height(Length::Shrink),
            ],
        ])
        .style(|_theme: &Theme| Style {
            background: Some(Color::from_rgba8(0, 0, 0, 0.4).into()),
            ..Default::default()
        })
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }
}

#[derive(Debug, Clone)]
enum Message {
    MainEditorMessage(MainEditorMessage),
    NewCellEditorMessage(NewCellEditorMessage),
}

#[derive(Debug, Clone)]
enum MainEditorMessage {
    NewCell,
    Edit(text_editor::Action),
    SelectCell(CellId),
    UpdateCell,
}

#[derive(Debug, Clone)]
enum NewCellEditorMessage {
    Edit(text_editor::Action),
    Submit,
    Cancel,
}
