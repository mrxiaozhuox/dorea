// Dorea CLI TUI Mode
// Terminal User Interface for Dorea Key-Value Storage System
//
// Author: YuKun Liu <mrxzx.info@gmail.com>

use dorea::client::DoreaClient;
use dorea::network::NetPacketState;
use dorea::value::DataValue;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Table, Row, Tabs, Clear},
    Frame, Terminal,
};
use std::io;
use std::time::Duration;

const DOREA_CLI_VERSION: &str = "0.5.0";

/// Tab 定义
#[derive(Debug, Clone, Copy, PartialEq)]
enum TabId {
    Data,
    Monitor,
    Log,
    Help,
}

impl TabId {
    fn titles() -> Vec<&'static str> {
        vec!["Data", "Monitor", "Log", "Help"]
    }
    
    fn next(self) -> Self {
        match self {
            TabId::Data => TabId::Monitor,
            TabId::Monitor => TabId::Log,
            TabId::Log => TabId::Help,
            TabId::Help => TabId::Data,
        }
    }
    
    fn prev(self) -> Self {
        match self {
            TabId::Data => TabId::Help,
            TabId::Monitor => TabId::Data,
            TabId::Log => TabId::Monitor,
            TabId::Help => TabId::Log,
        }
    }
}

/// 面板焦点
#[derive(Debug, Clone, Copy, PartialEq)]
enum Focus {
    DatabaseList,
    KeyList,
    ValueView,
    CommandInput,
}

/// 值显示模式
#[derive(Debug, Clone, Copy, PartialEq)]
enum ValueViewMode {
    Tree,
    Raw,
}

/// TUI 应用状态
pub struct App {
    /// 当前 Tab
    current_tab: TabId,
    /// 当前焦点面板
    focus: Focus,
    /// 值显示模式
    value_mode: ValueViewMode,
    
    // 连接信息
    hostname: String,
    port: u16,
    current_database: String,
    
    // 数据库列表
    databases: Vec<String>,
    selected_database: usize,
    
    // 键列表
    keys: Vec<KeyInfo>,
    selected_key: usize,
    key_scroll_offset: usize,
    
    // 当前值
    current_value: Option<String>,
    current_value_type: Option<String>,
    
    // 命令输入
    command_input: String,
    command_mode: bool,
    command_result: Option<(bool, String)>,
    
    // 操作日志
    logs: Vec<LogEntry>,
    
    // 状态消息
    status_message: String,
    
    // 是否应该退出
    should_quit: bool,
}

/// 键信息
#[derive(Debug, Clone)]
struct KeyInfo {
    key: String,
    key_type: String,
    size: String,
    ttl: String,
}

/// 日志条目
#[derive(Debug, Clone)]
struct LogEntry {
    timestamp: String,
    level: String,
    message: String,
}

impl App {
    pub fn new(hostname: String, port: u16) -> Self {
        Self {
            current_tab: TabId::Data,
            focus: Focus::DatabaseList,
            value_mode: ValueViewMode::Tree,
            hostname,
            port,
            current_database: "default".to_string(),
            databases: vec!["default".to_string()],
            selected_database: 0,
            keys: Vec::new(),
            selected_key: 0,
            key_scroll_offset: 0,
            current_value: None,
            current_value_type: None,
            command_input: String::new(),
            command_mode: false,
            command_result: None,
            logs: Vec::new(),
            status_message: "Press j/k to navigate, h/l to switch panels, q to quit".to_string(),
            should_quit: false,
        }
    }
    
    fn add_log(&mut self, level: &str, message: &str) {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        self.logs.push(LogEntry {
            timestamp,
            level: level.to_string(),
            message: message.to_string(),
        });
        // 保持最近 100 条日志
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }
}

/// 运行 TUI 模式
pub async fn run_tui(client: DoreaClient, hostname: String, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    // 设置终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // 创建应用状态
    let mut app = App::new(hostname, port);
    let mut client = client;
    
    // 初始加载数据库列表
    app.add_log("INFO", "Connected to server");
    
    // 主循环
    loop {
        // 绘制界面
        terminal.draw(|f| ui(f, &mut app))?;
        
        // 处理事件
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key_event(key, &mut app, &mut client).await?;
                }
            }
        }
        
        if app.should_quit {
            break;
        }
    }
    
    // 恢复终端
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    
    Ok(())
}

/// 处理键盘事件
async fn handle_key_event(
    key: event::KeyEvent,
    app: &mut App,
    client: &mut DoreaClient,
) -> Result<(), Box<dyn std::error::Error>> {
    // 命令输入模式
    if app.command_mode {
        match key.code {
            KeyCode::Esc => {
                app.command_mode = false;
                app.command_input.clear();
            }
            KeyCode::Enter => {
                // 执行命令
                let cmd = app.command_input.trim().to_string();
                if cmd == "q" || cmd == "quit" {
                    app.should_quit = true;
                } else {
                    execute_command(app, client, &cmd).await;
                }
                app.command_mode = false;
                app.command_input.clear();
            }
            KeyCode::Char(c) => {
                app.command_input.push(c);
            }
            KeyCode::Backspace => {
                app.command_input.pop();
            }
            _ => {}
        }
        return Ok(());
    }
    
    // 全局快捷键
    match key.code {
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Tab => {
            app.current_tab = app.current_tab.next();
        }
        KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::NONE) => {
            // gg = 跳转到顶部（需要两次 g）
        }
        KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.current_tab = app.current_tab.next();
        }
        KeyCode::Char(':') => {
            app.command_mode = true;
            app.command_input.clear();
        }
        _ => {
            // Tab 特定快捷键
            match app.current_tab {
                TabId::Data => handle_data_tab_keys(key, app, client).await?,
                TabId::Monitor => handle_monitor_tab_keys(key, app).await?,
                TabId::Log => handle_log_tab_keys(key, app).await?,
                TabId::Help => handle_help_tab_keys(key, app).await?,
            }
        }
    }
    
    Ok(())
}

/// 处理 Data Tab 快捷键
async fn handle_data_tab_keys(
    key: event::KeyEvent,
    app: &mut App,
    client: &mut DoreaClient,
) -> Result<(), Box<dyn std::error::Error>> {
    match key.code {
        // 导航
        KeyCode::Char('j') => {
            match app.focus {
                Focus::DatabaseList => {
                    if app.selected_database < app.databases.len() - 1 {
                        app.selected_database += 1;
                    }
                }
                Focus::KeyList => {
                    if app.selected_key < app.keys.len().saturating_sub(1) {
                        app.selected_key += 1;
                        // 加载值 - 先 clone key 避免借用冲突
                        let key = app.keys.get(app.selected_key).map(|k| k.key.clone());
                        if let Some(key) = key {
                            load_key_value(app, client, &key).await;
                        }
                    }
                }
                _ => {}
            }
        }
        KeyCode::Char('k') => {
            match app.focus {
                Focus::DatabaseList => {
                    if app.selected_database > 0 {
                        app.selected_database -= 1;
                    }
                }
                Focus::KeyList => {
                    if app.selected_key > 0 {
                        app.selected_key -= 1;
                        let key = app.keys.get(app.selected_key).map(|k| k.key.clone());
                        if let Some(key) = key {
                            load_key_value(app, client, &key).await;
                        }
                    }
                }
                _ => {}
            }
        }
        KeyCode::Char('h') => {
            match app.focus {
                Focus::KeyList => app.focus = Focus::DatabaseList,
                Focus::ValueView => app.focus = Focus::KeyList,
                _ => {}
            }
        }
        KeyCode::Char('l') => {
            match app.focus {
                Focus::DatabaseList => app.focus = Focus::KeyList,
                Focus::KeyList => app.focus = Focus::ValueView,
                _ => {}
            }
        }
        KeyCode::Char('G') => {
            if app.focus == Focus::KeyList && !app.keys.is_empty() {
                app.selected_key = app.keys.len() - 1;
            }
        }
        KeyCode::Enter => {
            if app.focus == Focus::DatabaseList {
                // 切换数据库
                if let Some(db) = app.databases.get(app.selected_database) {
                    let db_name = db.clone();
                    match client.select(&db_name).await {
                        Ok(_) => {
                            app.current_database = db_name.clone();
                            app.keys.clear();
                            app.selected_key = 0;
                            app.current_value = None;
                            app.add_log("INFO", &format!("Switched to database: {}", db_name));
                            // 加载键列表
                            load_keys(app, client).await;
                        }
                        Err(e) => {
                            app.add_log("ERROR", &format!("Failed to switch database: {:?}", e));
                        }
                    }
                }
            }
        }
        KeyCode::Char('r') => {
            // 刷新
            load_keys(app, client).await;
            app.add_log("INFO", "Refreshed key list");
        }
        KeyCode::Char('/') => {
            // TODO: 搜索
            app.status_message = "Search: (not implemented yet)".to_string();
        }
        KeyCode::F(2) => {
            // 切换 Tree/Raw 模式
            app.value_mode = match app.value_mode {
                ValueViewMode::Tree => ValueViewMode::Raw,
                ValueViewMode::Raw => ValueViewMode::Tree,
            };
        }
        _ => {}
    }
    
    Ok(())
}

/// 处理 Monitor Tab 快捷键
async fn handle_monitor_tab_keys(
    key: event::KeyEvent,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    match key.code {
        KeyCode::Char('r') => {
            app.add_log("INFO", "Refreshed monitor data");
        }
        _ => {}
    }
    Ok(())
}

/// 处理 Log Tab 快捷键
async fn handle_log_tab_keys(
    key: event::KeyEvent,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    match key.code {
        KeyCode::Char('j') => {
            // 向下滚动
        }
        KeyCode::Char('k') => {
            // 向上滚动
        }
        _ => {}
    }
    Ok(())
}

/// 处理 Help Tab 快捷键
async fn handle_help_tab_keys(
    key: event::KeyEvent,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

/// 加载键列表
async fn load_keys(app: &mut App, client: &mut DoreaClient) {
    match client.execute("info keys").await {
        Ok((state, data)) if state == NetPacketState::OK => {
            let result = String::from_utf8_lossy(&data).to_string();
            // 解析键列表 ["key1", "key2", ...]
            app.keys = parse_key_list(&result);
            app.status_message = format!("Loaded {} keys", app.keys.len());
            
            // 加载第一个键的值 - 先 clone key 避免借用冲突
            let first_key = app.keys.first().map(|k| k.key.clone());
            if let Some(key) = first_key {
                load_key_value(app, client, &key).await;
            }
        }
        _ => {
            app.keys.clear();
            app.status_message = "Failed to load keys".to_string();
        }
    }
}

/// 解析键列表
fn parse_key_list(data: &str) -> Vec<KeyInfo> {
    let mut keys = Vec::new();
    
    // 简单解析 JSON 数组
    let trimmed = data.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let content = &trimmed[1..trimmed.len()-1];
        for item in content.split(',') {
            let key = item.trim().trim_matches('"').to_string();
            if !key.is_empty() {
                keys.push(KeyInfo {
                    key: key.clone(),
                    key_type: "Unknown".to_string(),
                    size: "-".to_string(),
                    ttl: "-".to_string(),
                });
            }
        }
    }
    
    keys
}

/// 加载键的值
async fn load_key_value(app: &mut App, client: &mut DoreaClient, key: &str) {
    let command = format!("get {}", key);
    match client.execute(&command).await {
        Ok((state, data)) if state == NetPacketState::OK => {
            let value = String::from_utf8_lossy(&data).to_string();
            app.current_value = Some(value);
            // 尝试推断类型
            app.current_value_type = Some(infer_value_type(&app.current_value.as_ref().unwrap()));
        }
        _ => {
            app.current_value = Some("(error loading value)".to_string());
            app.current_value_type = None;
        }
    }
}

/// 推断值类型
fn infer_value_type(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        "Dict".to_string()
    } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
        "List".to_string()
    } else if trimmed.starts_with('(') && trimmed.ends_with(')') {
        "Tuple".to_string()
    } else if trimmed.starts_with('"') && trimmed.ends_with('"') {
        "String".to_string()
    } else if trimmed == "true" || trimmed == "false" {
        "Boolean".to_string()
    } else if trimmed.parse::<f64>().is_ok() {
        "Number".to_string()
    } else {
        "Unknown".to_string()
    }
}

/// 执行命令
async fn execute_command(app: &mut App, client: &mut DoreaClient, cmd: &str) {
    app.add_log("CMD", cmd);
    
    // 解析特殊命令
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }
    
    match parts[0] {
        "select" if parts.len() >= 2 => {
            let db = parts[1];
            match client.select(db).await {
                Ok(_) => {
                    app.current_database = db.to_string();
                    app.add_log("INFO", &format!("Switched to database: {}", db));
                }
                Err(e) => {
                    app.add_log("ERROR", &format!("Failed: {:?}", e));
                }
            }
        }
        _ => {
            // 执行通用命令
            match client.execute(cmd).await {
                Ok((state, data)) => {
                    let result = String::from_utf8_lossy(&data).to_string();
                    if state == NetPacketState::OK {
                        app.add_log("INFO", &format!("OK: {}", if result.len() > 50 { &result[..50] } else { &result }));
                    } else {
                        app.add_log("ERROR", &result);
                    }
                }
                Err(e) => {
                    app.add_log("ERROR", &format!("Error: {:?}", e));
                }
            }
        }
    }
}

/// 绘制 UI
fn ui(f: &mut Frame, app: &mut App) {
    // 创建主布局
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(1),  // 标题栏
            Constraint::Length(1),  // Tab 栏
            Constraint::Min(10),    // 主内容区
            Constraint::Length(1),  // 状态栏
        ])
        .split(f.size());
    
    // 标题栏 - 提前创建 String 避免临时值问题
    let port_str = app.port.to_string();
    let title = Paragraph::new(Line::from(vec![
        Span::styled(" Dorea CLI ", Style::default().fg(Color::Cyan).bold()),
        Span::raw("v"),
        Span::styled(DOREA_CLI_VERSION, Style::default().fg(Color::Green)),
        Span::raw("  "),
        Span::styled(&app.current_database, Style::default().fg(Color::Yellow).bold()),
        Span::raw("@"),
        Span::styled(&app.hostname, Style::default().fg(Color::White)),
        Span::raw(":"),
        Span::styled(&port_str, Style::default().fg(Color::White)),
    ]));
    f.render_widget(title, chunks[0]);
    
    // Tab 栏
    let titles: Vec<Line> = TabId::titles()
        .iter()
        .enumerate()
        .map(|(i, &title)| {
            if i == app.current_tab as usize {
                Line::from(Span::styled(
                    format!("[{}]", title),
                    Style::default().fg(Color::Cyan).bold(),
                ))
            } else {
                Line::from(Span::styled(
                    format!(" {} ", title),
                    Style::default().fg(Color::DarkGray),
                ))
            }
        })
        .collect();
    
    let tabs = Tabs::new(titles);
    f.render_widget(tabs, chunks[1]);
    
    // 主内容区
    match app.current_tab {
        TabId::Data => render_data_tab(f, app, chunks[2]),
        TabId::Monitor => render_monitor_tab(f, app, chunks[2]),
        TabId::Log => render_log_tab(f, app, chunks[2]),
        TabId::Help => render_help_tab(f, app, chunks[2]),
    }
    
    // 状态栏
    let status = Paragraph::new(Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(&app.status_message, Style::default().fg(Color::DarkGray)),
    ]));
    f.render_widget(status, chunks[3]);
    
    // 命令输入框
    if app.command_mode {
        let area = centered_rect(60, 3, f.size());
        f.render_widget(Clear, area);
        
        let input = Paragraph::new(Line::from(vec![
            Span::styled(":", Style::default().fg(Color::Yellow).bold()),
            Span::raw(&app.command_input),
            Span::styled("▌", Style::default().fg(Color::Cyan)),
        ]))
        .block(Block::default().borders(Borders::ALL).title("Command"));
        f.render_widget(input, area);
    }
}

/// 渲染 Data Tab
fn render_data_tab(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(40)])
        .split(area);
    
    // 左侧：数据库列表
    let db_items: Vec<ListItem> = app
        .databases
        .iter()
        .enumerate()
        .map(|(i, db)| {
            let style = if i == app.selected_database && app.focus == Focus::DatabaseList {
                Style::default().fg(Color::Yellow).bg(Color::DarkGray)
            } else if i == app.selected_database {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(Span::styled(format!("  {}", db), style)))
        })
        .collect();
    
    let db_list = List::new(db_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Databases ")
            .title_style(Style::default().fg(Color::Cyan)));
    f.render_widget(db_list, chunks[0]);
    
    // 右侧：键列表 + 值视图
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);
    
    // 键列表
    let key_rows: Vec<Row> = app
        .keys
        .iter()
        .enumerate()
        .map(|(i, key)| {
            let style = if i == app.selected_key && app.focus == Focus::KeyList {
                Style::default().fg(Color::Yellow).bg(Color::DarkGray)
            } else {
                Style::default()
            };
            Row::new(vec![
                Span::styled(&key.key, style.clone()),
                Span::styled(&key.key_type, style.clone()),
                Span::styled(&key.size, style.clone()),
            ])
        })
        .collect();
    
    let key_table = Table::new(
        key_rows,
        [Constraint::Percentage(50), Constraint::Percentage(25), Constraint::Percentage(25)],
    )
    .header(
        Row::new(vec![
            Span::styled("KEY", Style::default().fg(Color::Cyan).bold()),
            Span::styled("TYPE", Style::default().fg(Color::Cyan).bold()),
            Span::styled("SIZE", Style::default().fg(Color::Cyan).bold()),
        ])
    )
    .block(Block::default()
        .borders(Borders::ALL)
        .title(format!(" Keys ({}) ", app.keys.len()))
        .title_style(Style::default().fg(Color::Cyan)));
    f.render_widget(key_table, right_chunks[0]);
    
    // 值视图
    let value_title = if let Some(ref vtype) = app.current_value_type {
        format!(" Value ({}) ", vtype)
    } else {
        " Value ".to_string()
    };
    
    let mode_hint = match app.value_mode {
        ValueViewMode::Tree => " [F2: Raw]",
        ValueViewMode::Raw => " [F2: Tree]",
    };
    
    let value_content = match &app.current_value {
        Some(value) => value.clone(),
        None => "(no value)".to_string(),
    };
    
    let value_widget = Paragraph::new(value_content)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(value_title + mode_hint)
            .title_style(Style::default().fg(Color::Cyan)));
    f.render_widget(value_widget, right_chunks[1]);
}

/// 渲染 Monitor Tab
fn render_monitor_tab(f: &mut Frame, app: &mut App, area: Rect) {
    let text = vec![
        Line::from("Monitor Tab - Coming Soon"),
        Line::from(""),
        Line::from("This tab will show:"),
        Line::from("  - Server version"),
        Line::from("  - Connection count"),
        Line::from("  - Memory usage"),
        Line::from("  - Index count"),
        Line::from(""),
        Line::from("Auto-refresh every 2 seconds"),
    ];
    
    let paragraph = Paragraph::new(text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Monitor ")
            .title_style(Style::default().fg(Color::Cyan)));
    f.render_widget(paragraph, area);
}

/// 渲染 Log Tab
fn render_log_tab(f: &mut Frame, app: &mut App, area: Rect) {
    let log_items: Vec<ListItem> = app
        .logs
        .iter()
        .map(|log| {
            let level_style = match log.level.as_str() {
                "ERROR" => Style::default().fg(Color::Red),
                "WARN" => Style::default().fg(Color::Yellow),
                "INFO" => Style::default().fg(Color::Green),
                "CMD" => Style::default().fg(Color::Cyan),
                _ => Style::default().fg(Color::White),
            };
            
            ListItem::new(Line::from(vec![
                Span::styled(&log.timestamp, Style::default().fg(Color::DarkGray)),
                Span::raw(" "),
                Span::styled(format!("[{:^5}]", log.level), level_style),
                Span::raw(" "),
                Span::raw(&log.message),
            ]))
        })
        .collect();
    
    let log_list = List::new(log_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Logs ")
            .title_style(Style::default().fg(Color::Cyan)));
    f.render_widget(log_list, area);
}

/// 渲染 Help Tab
fn render_help_tab(f: &mut Frame, app: &mut App, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled("Dorea CLI TUI - Key Bindings", Style::default().fg(Color::Cyan).bold())),
        Line::from(""),
        Line::from(Span::styled("Global", Style::default().fg(Color::Yellow).bold())),
        Line::from("  q          Quit TUI"),
        Line::from("  Tab        Switch tab"),
        Line::from("  :          Command input"),
        Line::from(""),
        Line::from(Span::styled("Navigation (Vim Style)", Style::default().fg(Color::Yellow).bold())),
        Line::from("  j/k        Move up/down"),
        Line::from("  h/l        Switch panel (db → keys → value)"),
        Line::from("  G          Jump to bottom"),
        Line::from(""),
        Line::from(Span::styled("Data Tab", Style::default().fg(Color::Yellow).bold())),
        Line::from("  Enter      Select database"),
        Line::from("  r          Refresh key list"),
        Line::from("  F2         Toggle Tree/Raw mode"),
        Line::from(""),
        Line::from(Span::styled("Commands", Style::default().fg(Color::Yellow).bold())),
        Line::from("  :select <db>    Switch database"),
        Line::from("  :get <key>      Get value"),
        Line::from("  :set <k> <v>    Set value"),
        Line::from("  :q              Quit"),
    ];
    
    let paragraph = Paragraph::new(help_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Help ")
            .title_style(Style::default().fg(Color::Cyan)));
    f.render_widget(paragraph, area);
}

/// 创建居中矩形
fn centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height) / 2),
            Constraint::Length(height),
            Constraint::Percentage((100 - height) / 2),
        ])
        .split(r);
    
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
