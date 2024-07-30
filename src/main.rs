use std::io::{self, stdout};

use app::{handle_events, loading_ui, preload_app, ui, App};
use ratatui::{
    crossterm::{
        terminal::{
            disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
        },
        ExecutableCommand,
    },
    prelude::*
};
use save::SAVING;

mod app;
mod pos;
mod matrix;
mod save;

fn startup(frame: &mut Frame) {
    let layout = Layout::vertical(vec![Constraint::Percentage(100)]);
    let [a] = layout.areas(frame.size());
    loading_ui(frame, a);
}

#[tokio::main]
async fn main() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // 注册 Ctrl-C 的事件，防止非正常退出
    ctrlc::set_handler(|| {}).expect("Failed to set Ctrl-C handler");

    // 启动界面
    terminal.draw(|f| startup(f))?;

    // 创建 app 实例
    let mut app = App::new();

    // 预加载
    let saving = SAVING.lock().unwrap().clone();
    tokio::spawn(preload_app(saving));

    // 主循环
    while !app.should_exit {
        terminal.draw(|f| ui(f, &mut app))?;
        handle_events(&mut app);
    }

    // 退出
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
