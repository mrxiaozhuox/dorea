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
    KeyList,
    ValueView,
    CommandInput,
}

/// 值显示模式
#[derive(Debug, Clone, Copy, PartialEq)]
enum ValueViewMode {
    Pretty,
    Raw,
}

/// 值树节点（用于 Pretty 模式的组件化渲染）
#[derive(Debug, Clone)]
enum ValueNode {
    String(String),
    Number(String),
    Boolean(bool),
    Null,
    Array(Vec<ValueNode>),
    Object(Vec<(String, ValueNode)>),
}

/// 展开状态
#[derive(Debug, Clone, Default)]
struct ExpandState {
    expanded: std::collections::HashSet<String>,
    cursor_path: String,  // 当前光标所在路径
    scroll_offset: usize, // 值树滚动偏移
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
    
    // 键列表
    keys: Vec<KeyInfo>,
    selected_key: usize,
    key_scroll_offset: usize,
    
    // 当前值
    current_value: Option<String>,
    current_value_raw: Option<String>,
    current_value_type: Option<String>,
    
    // 值树（Pretty 模式用）
    value_tree: Option<ValueNode>,
    expand_state: ExpandState,
    
    // 命令输入
    command_input: String,
    command_mode: bool,
    command_result: Option<(bool, String)>,  // (success, message)
    command_result_time: Option<std::time::Instant>,  // 用于自动隐藏
    
    // 操作日志
    logs: Vec<LogEntry>,
    
    // Monitor 数据
    monitor: MonitorData,
    
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

/// Monitor 数据
#[derive(Debug, Clone, Default)]
struct MonitorData {
    server_version: String,
    uptime: String,
    connected_clients: String,
    total_keys: String,
    total_indexes: String,
    current_db: String,
    last_updated: String,
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
            focus: Focus::KeyList,
            value_mode: ValueViewMode::Pretty,
            hostname,
            port,
            current_database: "default".to_string(),
            keys: Vec::new(),
            selected_key: 0,
            key_scroll_offset: 0,
            current_value: None,
            current_value_raw: None,
            current_value_type: None,
            value_tree: None,
            expand_state: ExpandState::default(),
            command_input: String::new(),
            command_mode: false,
            command_result: None,
            command_result_time: None,
            logs: Vec::new(),
            monitor: MonitorData::default(),
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
    
    // 初始加载数据
    app.add_log("INFO", "Connected to server");
    load_keys(&mut app, &mut client).await;
    
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
        KeyCode::BackTab => {
            app.current_tab = app.current_tab.prev();
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
                TabId::Monitor => handle_monitor_tab_keys(key, app, client).await?,
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
        // 导航 - 支持 j/k 和 上下方向键
        KeyCode::Char('j') | KeyCode::Down => {
            if app.selected_key < app.keys.len().saturating_sub(1) {
                app.selected_key += 1;
                // 加载值 - 先 clone key 避免借用冲突
                let key = app.keys.get(app.selected_key).map(|k| k.key.clone());
                if let Some(key) = key {
                    load_key_value(app, client, &key).await;
                }
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.selected_key > 0 {
                app.selected_key -= 1;
                let key = app.keys.get(app.selected_key).map(|k| k.key.clone());
                if let Some(key) = key {
                    load_key_value(app, client, &key).await;
                }
            }
        }
        // 支持 h/l 和 左右方向键
        KeyCode::Char('h') | KeyCode::Left => {
            if app.focus == Focus::ValueView {
                app.focus = Focus::KeyList;
            }
        }
        KeyCode::Char('l') | KeyCode::Right => {
            if app.focus == Focus::KeyList {
                app.focus = Focus::ValueView;
            }
        }
        KeyCode::Char('G') => {
            if !app.keys.is_empty() {
                app.selected_key = app.keys.len() - 1;
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
        // 切换 Pretty/Raw 模式
        KeyCode::F(2) => {
            app.value_mode = match app.value_mode {
                ValueViewMode::Pretty => ValueViewMode::Raw,
                ValueViewMode::Raw => ValueViewMode::Pretty,
            };
        }
        // Enter 键展开/折叠值树节点
        KeyCode::Enter => {
            if app.focus == Focus::ValueView && app.value_mode == ValueViewMode::Pretty {
                // 切换根节点展开状态
                if app.expand_state.expanded.contains("") {
                    app.expand_state.expanded.remove("");
                } else {
                    app.expand_state.expanded.insert("".to_string());
                }
            }
        }
        _ => {}
    }
    
    Ok(())
}

/// 处理 Monitor Tab 快捷键
async fn handle_monitor_tab_keys(
    key: event::KeyEvent,
    app: &mut App,
    client: &mut DoreaClient,
) -> Result<(), Box<dyn std::error::Error>> {
    match key.code {
        KeyCode::Char('r') => {
            load_monitor_data(app, client).await;
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
            // 解析键列表，保留已有 key 的缓存
            let old_keys = std::mem::take(&mut app.keys);
            app.keys = parse_key_list(&result, &old_keys);
            
            // 尝试保持选中状态
            if app.selected_key >= app.keys.len() {
                app.selected_key = app.keys.len().saturating_sub(1);
            }
            
            app.status_message = format!("Loaded {} keys", app.keys.len());
            
            // 加载当前选中键的值 - 先 clone key 避免借用冲突
            let selected_key = app.keys.get(app.selected_key).map(|k| k.key.clone());
            if let Some(key) = selected_key {
                load_key_value(app, client, &key).await;
            }
        }
        _ => {
            app.keys.clear();
            app.status_message = "Failed to load keys".to_string();
        }
    }
}

/// 解析键列表，保留已有 key 的缓存
fn parse_key_list(data: &str, cached_keys: &[KeyInfo]) -> Vec<KeyInfo> {
    let mut keys = Vec::new();
    
    // 简单解析 JSON 数组
    let trimmed = data.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        let content = &trimmed[1..trimmed.len()-1];
        for item in content.split(',') {
            let key = item.trim().trim_matches('"').to_string();
            if !key.is_empty() {
                // 查找缓存
                let cached = cached_keys.iter().find(|k| k.key == key);
                keys.push(KeyInfo {
                    key: key.clone(),
                    key_type: cached.map(|k| k.key_type.clone()).unwrap_or_else(|| "-".to_string()),
                    size: cached.map(|k| k.size.clone()).unwrap_or_else(|| "-".to_string()),
                    ttl: cached.map(|k| k.ttl.clone()).unwrap_or_else(|| "-".to_string()),
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
            let raw_value = String::from_utf8_lossy(&data).to_string();
            
            // 检查是否为空或 key 不存在
            if raw_value.is_empty() {
                app.current_value = Some("(key not found)".to_string());
                app.current_value_raw = None;
                app.current_value_type = None;
                return;
            }
            
            let value_type = infer_value_type(&raw_value);
            
            // 解析值树（用于 Pretty 模式的组件化渲染）
            let value_tree = parse_value_to_tree(&raw_value);
            
            // 格式化值（作为备用的简单文本）
            let formatted_value = format_value_simple(&raw_value, &value_type);
            
            app.current_value_raw = Some(raw_value.clone());
            app.current_value = Some(formatted_value);
            app.current_value_type = Some(value_type.clone());
            app.value_tree = value_tree;
            app.expand_state = ExpandState::default();  // 重置展开状态
            
            // 更新键列表中的类型
            if let Some(key_info) = app.keys.iter_mut().find(|k| k.key == key) {
                key_info.key_type = value_type;
                key_info.size = format!("{}B", raw_value.len());
            }
        }
        _ => {
            app.current_value = Some("(error loading value)".to_string());
            app.current_value_raw = None;
            app.current_value_type = None;
            app.value_tree = None;
        }
    }
}

/// 解析 JSON 值为 ValueNode 树
fn parse_value_to_tree(value: &str) -> Option<ValueNode> {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(value) {
        Some(json_to_node(&json))
    } else {
        None
    }
}

/// JSON Value 转 ValueNode
fn json_to_node(value: &serde_json::Value) -> ValueNode {
    match value {
        serde_json::Value::String(s) => ValueNode::String(s.clone()),
        serde_json::Value::Number(n) => ValueNode::Number(n.to_string()),
        serde_json::Value::Bool(b) => ValueNode::Boolean(*b),
        serde_json::Value::Null => ValueNode::Null,
        serde_json::Value::Array(arr) => {
            ValueNode::Array(arr.iter().map(json_to_node).collect())
        }
        serde_json::Value::Object(map) => {
            ValueNode::Object(
                map.iter()
                    .map(|(k, v)| (k.clone(), json_to_node(v)))
                    .collect()
            )
        }
    }
}

/// 渲染值树为可滚动的 Lines
fn render_value_tree(node: &ValueNode, expand_state: &ExpandState) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    render_node(node, "", 0, true, expand_state, &mut lines);
    if lines.is_empty() {
        lines.push(Line::from("(empty)"));
    }
    lines
}

/// 递归渲染节点
fn render_node(
    node: &ValueNode,
    path: &str,
    depth: usize,
    is_last: bool,
    expand_state: &ExpandState,
    lines: &mut Vec<Line<'static>>,
) {
    let indent = "  ".repeat(depth);
    let prefix = if depth == 0 { "" } else if is_last { "└─" } else { "├─" };
    
    match node {
        ValueNode::String(s) => {
            let display = if s.len() > 50 {
                format!("\"{}...\"", &s[..47])
            } else {
                format!("\"{}\"", s)
            };
            lines.push(Line::from(vec![
                Span::raw(format!("{}{}", indent, prefix)),
                Span::styled(display, Style::default().fg(Color::Green)),
            ]));
        }
        ValueNode::Number(n) => {
            lines.push(Line::from(vec![
                Span::raw(format!("{}{}", indent, prefix)),
                Span::styled(n.clone(), Style::default().fg(Color::Yellow)),
            ]));
        }
        ValueNode::Boolean(b) => {
            let text = if *b { "true" } else { "false" };
            lines.push(Line::from(vec![
                Span::raw(format!("{}{}", indent, prefix)),
                Span::styled(text, Style::default().fg(Color::Magenta)),
            ]));
        }
        ValueNode::Null => {
            lines.push(Line::from(vec![
                Span::raw(format!("{}{}", indent, prefix)),
                Span::styled("null", Style::default().fg(Color::DarkGray)),
            ]));
        }
        ValueNode::Array(arr) => {
            let is_expanded = expand_state.expanded.contains(path);
            let count = arr.len();
            let icon = if is_expanded { "▼" } else { "▶" };
            
            lines.push(Line::from(vec![
                Span::raw(format!("{}{}", indent, prefix)),
                Span::styled(format!("{} [{} items]", icon, count), Style::default().fg(Color::Cyan)),
            ]));
            
            if is_expanded {
                for (i, item) in arr.iter().enumerate() {
                    let item_path = format!("{}[{}]", path, i);
                    let item_is_last = i == arr.len() - 1;
                    render_node(item, &item_path, depth + 1, item_is_last, expand_state, lines);
                }
            }
        }
        ValueNode::Object(map) => {
            let is_expanded = expand_state.expanded.contains(path);
            let count = map.len();
            let icon = if is_expanded { "▼" } else { "▶" };
            
            lines.push(Line::from(vec![
                Span::raw(format!("{}{}", indent, prefix)),
                Span::styled(format!("{} {{{}}} keys", icon, count), Style::default().fg(Color::Cyan)),
            ]));
            
            if is_expanded {
                for (i, (key, value)) in map.iter().enumerate() {
                    let item_path = format!("{}.{}", path, key);
                    let item_is_last = i == map.len() - 1;
                    
                    // 先渲染 key
                    let key_indent = "  ".repeat(depth + 1);
                    let key_prefix = if item_is_last { "└─" } else { "├─" };
                    
                    match value {
                        ValueNode::String(s) => {
                            let display = if s.len() > 40 {
                                format!("\"{}...\"", &s[..37])
                            } else {
                                format!("\"{}\"", s)
                            };
                            lines.push(Line::from(vec![
                                Span::raw(format!("{}{}", key_indent, key_prefix)),
                                Span::styled(key.clone(), Style::default().fg(Color::Blue)),
                                Span::raw(": "),
                                Span::styled(display, Style::default().fg(Color::Green)),
                            ]));
                        }
                        ValueNode::Number(n) => {
                            lines.push(Line::from(vec![
                                Span::raw(format!("{}{}", key_indent, key_prefix)),
                                Span::styled(key.clone(), Style::default().fg(Color::Blue)),
                                Span::raw(": "),
                                Span::styled(n.clone(), Style::default().fg(Color::Yellow)),
                            ]));
                        }
                        ValueNode::Boolean(b) => {
                            let text = if *b { "true" } else { "false" };
                            lines.push(Line::from(vec![
                                Span::raw(format!("{}{}", key_indent, key_prefix)),
                                Span::styled(key.clone(), Style::default().fg(Color::Blue)),
                                Span::raw(": "),
                                Span::styled(text, Style::default().fg(Color::Magenta)),
                            ]));
                        }
                        ValueNode::Null => {
                            lines.push(Line::from(vec![
                                Span::raw(format!("{}{}", key_indent, key_prefix)),
                                Span::styled(key.clone(), Style::default().fg(Color::Blue)),
                                Span::raw(": "),
                                Span::styled("null", Style::default().fg(Color::DarkGray)),
                            ]));
                        }
                        _ => {
                            // 复合类型，递归
                            lines.push(Line::from(vec![
                                Span::raw(format!("{}{}", key_indent, key_prefix)),
                                Span::styled(key.clone(), Style::default().fg(Color::Blue)),
                                Span::raw(": "),
                            ]));
                            render_node(value, &item_path, depth + 2, item_is_last, expand_state, lines);
                        }
                    }
                }
            }
        }
    }
}

/// 格式化值（Pretty 模式）- 简单文本美化（非 JSON）
fn format_value_simple(value: &str, value_type: &str) -> String {
    match value_type {
        "Dict" | "List" | "Tuple" => {
            // 尝试 JSON 美化
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(value) {
                serde_json::to_string_pretty(&json).unwrap_or_else(|_| value.to_string())
            } else {
                value.to_string()
            }
        }
        _ => value.to_string(),
    }
}

/// 推断值类型
fn infer_value_type(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.starts_with("binary!(") {
        "Binary".to_string()
    } else if trimmed.starts_with('{') && trimmed.ends_with('}') {
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
        "String".to_string()
    }
}

/// 加载 Monitor 数据
async fn load_monitor_data(app: &mut App, client: &mut DoreaClient) {
    // 获取服务器版本 - 直接返回版本字符串
    if let Ok((state, data)) = client.execute("info version").await {
        if state == NetPacketState::OK {
            let version = String::from_utf8_lossy(&data).to_string();
            app.monitor.server_version = version.trim().trim_matches('"').to_string();
        }
    }
    
    // 获取服务器启动时间 - 返回时间戳字符串
    if let Ok((state, data)) = client.execute("info server-startup-time").await {
        if state == NetPacketState::OK {
            let startup = String::from_utf8_lossy(&data).to_string();
            // 尝试解析时间戳计算 uptime
            if let Ok(ts) = startup.trim().parse::<i64>() {
                let now = chrono::Utc::now().timestamp();
                let uptime_secs = (now - ts).max(0) as u64;
                app.monitor.uptime = format_seconds(uptime_secs);
            }
        }
    }
    
    // 获取当前连接数
    if let Ok((state, data)) = client.execute("info current-connect-num").await {
        if state == NetPacketState::OK {
            let count = String::from_utf8_lossy(&data).to_string();
            app.monitor.connected_clients = count.trim().to_string();
        }
    }
    
    // 获取键数量
    if let Ok((state, data)) = client.execute("info keys").await {
        if state == NetPacketState::OK {
            let result = String::from_utf8_lossy(&data).to_string();
            // 解析键列表 ["key1", "key2", ...]
            if result.starts_with('[') {
                let count = result.matches(',').count().max(0) + if result.contains('"') { 1 } else { 0 };
                app.monitor.total_keys = count.to_string();
            }
        }
    }
    
    // 获取总索引数
    if let Ok((state, data)) = client.execute("info total-index-number").await {
        if state == NetPacketState::OK {
            let count = String::from_utf8_lossy(&data).to_string();
            app.monitor.total_indexes = count.trim().to_string();
        }
    }
    
    // 获取当前数据库
    if let Ok((state, data)) = client.execute("info current").await {
        if state == NetPacketState::OK {
            let db = String::from_utf8_lossy(&data).to_string();
            app.monitor.current_db = db.trim().trim_matches('"').to_string();
        }
    }
    
    // 更新时间
    app.monitor.last_updated = chrono::Local::now().format("%H:%M:%S").to_string();
}

/// 格式化秒数为可读时间
fn format_seconds(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
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
    
    let show_result = |app: &mut App, success: bool, msg: &str| {
        app.command_result = Some((success, msg.to_string()));
        app.command_result_time = Some(std::time::Instant::now());
    };
    
    match parts[0] {
        "select" if parts.len() >= 2 => {
            let db = parts[1];
            match client.select(db).await {
                Ok(_) => {
                    app.current_database = db.to_string();
                    app.keys.clear();  // 清空缓存
                    app.selected_key = 0;
                    app.current_value = None;
                    app.current_value_raw = None;
                    show_result(app, true, &format!("Switched to database: {}", db));
                    app.add_log("INFO", &format!("Switched to database: {}", db));
                    // 切换数据库后刷新键列表
                    load_keys(app, client).await;
                }
                Err(e) => {
                    show_result(app, false, &format!("Error: {:?}", e));
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
                        let preview = if result.len() > 100 { 
                            format!("{}...", &result[..97])
                        } else { 
                            result.clone()
                        };
                        show_result(app, true, &preview);
                        app.add_log("INFO", &format!("OK: {}", preview));
                        // 执行命令后自动刷新键列表（保留缓存）
                        load_keys(app, client).await;
                    } else {
                        show_result(app, false, &result);
                        app.add_log("ERROR", &result);
                    }
                }
                Err(e) => {
                    show_result(app, false, &format!("Error: {:?}", e));
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
    
    // 命令结果弹出窗口 (显示 3 秒后自动消失)
    if let Some((success, ref msg)) = app.command_result {
        if let Some(time) = app.command_result_time {
            if time.elapsed().as_secs() < 3 {
                let area = centered_rect(70, 3, f.size());
                f.render_widget(Clear, area);
                
                let (icon, color) = if success {
                    ("✓", Color::Green)
                } else {
                    ("✗", Color::Red)
                };
                
                let result = Paragraph::new(Line::from(vec![
                    Span::styled(format!("{} ", icon), Style::default().fg(color).bold()),
                    Span::styled(msg, Style::default().fg(Color::White)),
                ]))
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(" Result ")
                    .title_style(Style::default().fg(color))
                    .border_style(Style::default().fg(color)));
                f.render_widget(result, area);
            } else {
                // 超时自动清除
                app.command_result = None;
                app.command_result_time = None;
            }
        }
    }
}

/// 渲染 Data Tab
fn render_data_tab(f: &mut Frame, app: &mut App, area: Rect) {
    // 两列布局：键列表 | 值视图
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);
    
    // 左侧：键列表
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
    f.render_widget(key_table, chunks[0]);
    
    // 右侧：值视图
    let value_title = if let Some(ref vtype) = app.current_value_type {
        format!(" Value ({}) ", vtype)
    } else {
        " Value ".to_string()
    };
    
    let mode_hint = match app.value_mode {
        ValueViewMode::Pretty => " [F2: Raw]",
        ValueViewMode::Raw => " [F2: Pretty]",
    };
    
    // 根据模式选择显示的值
    match app.value_mode {
        ValueViewMode::Pretty => {
            // 使用组件化渲染
            if let Some(tree) = &app.value_tree {
                let lines = render_value_tree(tree, &app.expand_state);
                let value_widget = Paragraph::new(lines)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title(value_title.clone() + mode_hint)
                        .title_style(Style::default().fg(Color::Cyan)));
                f.render_widget(value_widget, chunks[1]);
            } else {
                // 没有值树，显示简单文本
                let text = app.current_value.as_deref().unwrap_or("(no value)");
                let value_widget = Paragraph::new(text)
                    .block(Block::default()
                        .borders(Borders::ALL)
                        .title(value_title.clone() + mode_hint)
                        .title_style(Style::default().fg(Color::Cyan)));
                f.render_widget(value_widget, chunks[1]);
            }
        }
        ValueViewMode::Raw => {
            // 显示原始值
            let text = app.current_value_raw.as_deref().unwrap_or("(no value)");
            let value_widget = Paragraph::new(text)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .title(value_title + mode_hint)
                    .title_style(Style::default().fg(Color::Cyan)));
            f.render_widget(value_widget, chunks[1]);
        }
    }
}

/// 渲染 Monitor Tab
fn render_monitor_tab(f: &mut Frame, app: &mut App, area: Rect) {
    // 使用三行布局：顶部标题、中间信息、底部提示
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // 标题行
            Constraint::Min(10),    // 信息区
            Constraint::Length(3),  // 提示行
        ])
        .split(area);
    
    // 顶部标题 - 显示连接信息和当前数据库
    let title_text = format!(
        " {}:{}  │  DB: {} ",
        app.hostname,
        app.port,
        if app.monitor.current_db.is_empty() { &app.current_database } else { &app.monitor.current_db }
    );
    let title = Paragraph::new(title_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);
    
    // 中间信息区 - 两列布局
    let info_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);
    
    // 左侧 - 服务器信息
    let server_version = if app.monitor.server_version.is_empty() { "-" } else { &app.monitor.server_version };
    let uptime = if app.monitor.uptime.is_empty() { "-" } else { &app.monitor.uptime };
    let connected = if app.monitor.connected_clients.is_empty() { "-" } else { &app.monitor.connected_clients };
    
    let server_info = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  ● ", Style::default().fg(Color::Green)),
            Span::styled("Server Version", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::raw("    "),
            Span::styled(server_version, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ⏱ ", Style::default().fg(Color::Yellow)),
            Span::styled("Uptime", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::raw("    "),
            Span::styled(uptime, Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ↻ ", Style::default().fg(Color::Cyan)),
            Span::styled("Connections", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::raw("    "),
            Span::styled(connected, Style::default().fg(Color::Green)),
        ]),
    ];
    
    let left_panel = Paragraph::new(server_info)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Server ")
            .title_style(Style::default().fg(Color::Yellow)));
    f.render_widget(left_panel, info_chunks[0]);
    
    // 右侧 - 数据统计
    let total_keys = if app.monitor.total_keys.is_empty() { "-" } else { &app.monitor.total_keys };
    let total_indexes = if app.monitor.total_indexes.is_empty() { "-" } else { &app.monitor.total_indexes };
    let last_updated = if app.monitor.last_updated.is_empty() { "-" } else { &app.monitor.last_updated };
    
    let data_stats = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  🔑 ", Style::default().fg(Color::Magenta)),
            Span::styled("Total Keys", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::raw("    "),
            Span::styled(total_keys, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  📊 ", Style::default().fg(Color::Blue)),
            Span::styled("Total Indexes", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::raw("    "),
            Span::styled(total_indexes, Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  🕐 ", Style::default().fg(Color::DarkGray)),
            Span::styled("Last Update", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::raw("    "),
            Span::styled(last_updated, Style::default().fg(Color::Yellow)),
        ]),
    ];
    
    let right_panel = Paragraph::new(data_stats)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Stats ")
            .title_style(Style::default().fg(Color::Cyan)));
    f.render_widget(right_panel, info_chunks[1]);
    
    // 底部提示
    let hint = Paragraph::new(" Press [r] to refresh  │  [Tab] switch tab  │  [q] quit ")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(hint, chunks[2]);
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
        Line::from("  q              Quit TUI"),
        Line::from("  Tab            Next tab"),
        Line::from("  Shift+Tab      Previous tab"),
        Line::from("  :              Command input"),
        Line::from(""),
        Line::from(Span::styled("Navigation", Style::default().fg(Color::Yellow).bold())),
        Line::from("  j / ↓          Move down"),
        Line::from("  k / ↑          Move up"),
        Line::from("  h / ←          Previous panel"),
        Line::from("  l / →          Next panel"),
        Line::from("  G              Jump to bottom"),
        Line::from("  g              Jump to top"),
        Line::from(""),
        Line::from(Span::styled("Data Tab", Style::default().fg(Color::Yellow).bold())),
        Line::from("  Enter          Load selected key value"),
        Line::from("  r              Refresh key list"),
        Line::from("  F2             Toggle Pretty/Raw mode"),
        Line::from(""),
        Line::from(Span::styled("Commands", Style::default().fg(Color::Yellow).bold())),
        Line::from("  :select <db>   Switch database"),
        Line::from("  :get <key>     Get value"),
        Line::from("  :set <k> <v>   Set value"),
        Line::from("  :del <key>     Delete key"),
        Line::from("  :q             Quit"),
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
