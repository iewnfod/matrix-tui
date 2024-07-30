use std::collections::HashMap;

use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
    prelude::*,
    widgets::*,
};
use strum::EnumIter;

use crate::{matrix::{self, get_matrix_client}, pos::{get_nearest_focus_area, get_top_left_focus_area}, save::Saving};

const FOCUSED_COLOR: ratatui::prelude::Color = Color::Yellow;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
enum InputMode {
    #[default] Normal,
    Editing
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
enum CurrentScreen {
    #[default] Login,
    Main,
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, EnumIter)]
pub enum FocusArea {
    #[default] ServerInput,
    UsernameInput,
    PasswordInput,
    LoginBt
}

pub fn handle_events(app: &mut App) {
    if event::poll(std::time::Duration::from_millis(50)).unwrap_or_default() {
        if let Event::Key(key) = event::read().unwrap_or(Event::FocusLost) {
            if key.kind == event::KeyEventKind::Press {
                // app.add_info = format!("{:?} {:?}", key.modifiers, key.code);
                match app.input_mode {
                    InputMode::Normal => {
                        match key.code {
                            KeyCode::Char('q') => {
                                app.should_exit = true;
                            },
                            KeyCode::Char('i') => {
                                app.input_mode = InputMode::Editing;
                                app.move_cursor_rightest();
                            },
                            KeyCode::Enter => {
                                app.click_focus();
                            },
                            KeyCode::Down | KeyCode::Up | KeyCode::Left | KeyCode::Right => {
                                if let Some(focus_area) = get_nearest_focus_area(&app, key.code) {
                                    app.current_focus = focus_area;
                                }
                            },
                            KeyCode::Tab => {
                                if let Some(down_focus_area) = get_nearest_focus_area(&app, KeyCode::Down) {
                                    app.current_focus = down_focus_area;
                                } else {
                                    if let Some(right_focus_area) = get_nearest_focus_area(&app, KeyCode::Right) {
                                        app.current_focus = right_focus_area;
                                    } else {
                                        // back to top
                                        if let Some(top_left_focus_area) = get_top_left_focus_area(&app) {
                                            app.current_focus = top_left_focus_area;
                                        }
                                    }
                                }
                            },
                            _ => {}
                        }
                    },
                    InputMode::Editing => {
                        match key.code {
                            KeyCode::Enter => app.input_mode = InputMode::Normal,
                            KeyCode::Char(to_insert) => {
                                app.enter_char(to_insert);
                            },
                            KeyCode::Backspace => {
                                app.delete_char();
                            },
                            KeyCode::Left => {
                                app.move_cursor_left();
                            },
                            KeyCode::Right => {
                                app.move_cursor_right();
                            },
                            KeyCode::Esc => {
                                app.input_mode = InputMode::Normal;
                            },
                            _ => {}
                        };
                        match key.modifiers {
                            KeyModifiers::CONTROL => {
                                match key.code {
                                    KeyCode::Char('u') | KeyCode::Char('h') => {
                                        app.clear_current_content();
                                    },
                                    _ => {}
                                }
                            },
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

fn one_line_input_block<'a>(app: &mut App, area: FocusArea, rect: Rect, block: Block<'a>, frame: &mut Frame) {
    let mut s = Style::default();
    let mut inner_value = match app.input_data.get(&area) {
        Some(t) => t.to_string(),
        None => "".to_string()
    };

    if area == app.current_focus {
        s = s.fg(FOCUSED_COLOR);
        if app.input_mode == InputMode::Editing {
            frame.set_cursor(rect.x + 2 + app.char_index as u16, rect.y + 1);
        }
    }

    if area == FocusArea::PasswordInput {
        let mut new_value = String::new();
        for _ in 0..inner_value.len() {
            new_value.push('*');
        }
        inner_value = new_value;
    }

    let inner_area = Paragraph::new(inner_value)
        .style(s)
        .block(block.padding(Padding::horizontal(1)));

    app.focus_area_positions.insert(area.clone(), rect);

    frame.render_widget(inner_area, rect);
}

pub fn ui(frame: &mut Frame, app: &mut App) {
    // 清空临时辅助数据
    app.focus_area_positions.clear();

    // 获取 Matrix 数据
    let client = get_matrix_client();
    let mut is_add_info_error = false;
    app.add_info = client.info_message;
    if !client.error_message.is_empty() {
        app.add_info = client.error_message;
        is_add_info_error = true;
    }

    if client.connected && app.current_screen != CurrentScreen::Main {
        app.current_screen = CurrentScreen::Main;
    }

    // 布局
    let layout = Layout::vertical(vec![
        Constraint::Min(1),
        Constraint::Length(1),
        Constraint::Length(1)
    ]).margin(1);

    let [main_area, info_area, help_area] = layout.areas(frame.size());

    frame.render_widget(
        Text::raw(&app.add_info)
            .centered()
            .style(Style::default().fg({
                if is_add_info_error {
                    Color::Red
                } else {
                    Color::Gray
                }
            }).bold()),
        info_area
    );

    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                "Press ".into(),
                "<q>".bold(),
                " to exit, ".into(),
                "<i>".bold(),
                " to start editing, ".into(),
                "<Enter>".bold(),
                " to select, ".into(),
                "↑ ↓ ← →".bold(),
                " to control focus.".into()
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Editing => (
            vec![
                "Press ".into(),
                "<Esc>".bold(),
                " to stop editing, ".into(),
                "<Enter>".bold(),
                " to send message or confirm.".into(),
            ],
            Style::default(),
        ),
    };

    let help_text = Text::from(Line::from(msg)).patch_style(style).centered();
    frame.render_widget(help_text, help_area);

    if client.loading {
        loading_ui(frame, main_area);
        return;
    }

    match app.current_screen {
        CurrentScreen::Login => {
            let layout = Layout::vertical(vec![
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3)
            ])
                .flex(layout::Flex::SpaceBetween)
                .horizontal_margin(10)
                .vertical_margin(1);

            let [
                text_area,
                server_area,
                username_area,
                password_area,
                login_button_area
            ] = layout.areas(main_area);

            let login_text = Text::from("Login to your matrix account".bold()).alignment(Alignment::Center);
            frame.render_widget(login_text, text_area);

            one_line_input_block(
                app,
                FocusArea::ServerInput,
                server_area,
                Block::bordered().title(" Server Address "),
                frame
            );

            one_line_input_block(
                app,
                FocusArea::UsernameInput,
                username_area,
                Block::bordered().title(" Username "),
                frame
            );

            one_line_input_block(
                app,
                FocusArea::PasswordInput,
                password_area,
                Block::bordered().title(" Password "),
                frame
            );

            let login_bt = Paragraph::new("Login".bold())
                .centered().block(Block::bordered())
                .style(if app.current_focus == FocusArea::LoginBt {
                    Style::default().fg(FOCUSED_COLOR)
                } else {
                    Style::default()
                });
            let [login_bt_layout] = Layout::horizontal(
                vec![Constraint::Percentage(10)]
            ).flex(layout::Flex::Center).areas(login_button_area);
            app.focus_area_positions.insert(FocusArea::LoginBt, login_bt_layout);
            frame.render_widget(login_bt, login_bt_layout);
        },
        CurrentScreen::Main => {
            frame.render_widget(Text::from("Main Screen"), main_area);
        },
    }
}

#[derive(Debug, Default, Clone)]
pub struct App {
    input_mode: InputMode,
    current_screen: CurrentScreen,
    char_index: usize,
    pub should_exit: bool,
    pub input_data: HashMap<FocusArea, String>,
    pub current_focus: FocusArea,
    pub add_info: String,
    pub focus_area_positions: HashMap<FocusArea, Rect>,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self::default();
        app.should_exit = false;
        app
    }

    fn get_current_value(&self) -> &str {
        if let Some(t) = self.input_data.get(&self.current_focus) {
            t.as_str()
        } else { "" }
    }

    fn enter_char(&mut self, c: char) {
        if let Some(t) = self.input_data.get_mut(&self.current_focus) {
            t.push(c);
        } else {
            self.input_data.insert(self.current_focus.clone(), c.to_string());
        }
        self.move_cursor_right();
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.char_index.saturating_sub(1);
        self.char_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.char_index.saturating_add(1);
        self.char_index = self.clamp_cursor(cursor_moved_right);
    }

    fn delete_char(&mut self) {
        if self.char_index != 0 {
            let current_index = self.char_index;
            let from_left_to_current_index = current_index - 1;
            let t = self.get_current_value();

            let before_char_to_delete = t.chars().take(from_left_to_current_index);
            let after_char_to_delete = t.chars().skip(current_index);

            let new_value: String = before_char_to_delete.chain(after_char_to_delete).collect();
            self.input_data.insert(self.current_focus.clone(), new_value);

            self.move_cursor_left();
        }
    }

    fn reset_cursor(&mut self) {
        self.char_index = 0;
    }

    fn clear_current_content(&mut self) {
        self.input_data.remove(&self.current_focus);
        self.reset_cursor();
    }

    fn move_cursor_rightest(&mut self) {
        self.char_index = self.get_current_value().chars().count();
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.get_current_value().chars().count())
    }

    fn get_input_data(&self, key: &FocusArea) -> String {
        if let Some(data) = self.input_data.get(key) {
            data.clone()
        } else {
            String::new()
        }
    }

    fn click_focus(&mut self) {
        match self.current_focus {
            FocusArea::LoginBt => {
                let server = self.get_input_data(&FocusArea::ServerInput);
                let username = self.get_input_data(&FocusArea::UsernameInput);
                let password = self.get_input_data(&FocusArea::PasswordInput);

                tokio::spawn(async {
                    matrix::login(server, username, password).await;
                });
            },
            _ => {}
        }
    }
}

pub fn loading_ui(frame: &mut Frame, area: Rect) {
    let centered_layout = Layout::vertical(
        [Constraint::Length(1)]
    ).flex(layout::Flex::Center);
    let [center_area] = centered_layout.areas(area);
    frame.render_widget(
        Paragraph::new("Loading...".bold().italic()).centered(),
        center_area,
    );
}

pub async fn preload_app(saving: Saving) {
    if !saving.token.is_empty() {
        matrix::login_with_token(&saving.server, &saving.token).await;
    }
}
