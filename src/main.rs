use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use serde::Deserialize;
use std::{
    fs,
    io,
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::Duration,
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum SortMode {
    Name,
    LastPlayed,
    Playtime,
}

impl SortMode {
    fn next(&self) -> Self {
        match self {
            SortMode::Name => SortMode::LastPlayed,
            SortMode::LastPlayed => SortMode::Playtime,
            SortMode::Playtime => SortMode::Name,
        }
    }

    fn display(&self) -> &str {
        match self {
            SortMode::Name => "Name",
            SortMode::LastPlayed => "Last Played",
            SortMode::Playtime => "Playtime",
        }
    }
}

#[derive(Debug, Clone)]
struct Instance {
    name: String,
    path: PathBuf,
    last_played: Option<String>,
    last_played_ts: Option<u64>,
    time_played: Option<String>,
    time_played_secs: Option<u64>,
    mc_version: Option<String>,
}

fn is_instance_running(instance_name: &str) -> bool {
    // Check if there's a flatpak process running with this instance name
    if let Ok(output) = Command::new("ps")
        .args(&["aux"])
        .output()
    {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            // Look for flatpak processes with the instance name in the command
            return stdout.lines().any(|line| {
                line.contains("flatpak") &&
                line.contains("PrismLauncher") &&
                line.contains(instance_name)
            });
        }
    }
    false
}

#[derive(Debug, Deserialize)]
struct InstanceConfig {
    #[serde(rename = "General")]
    general: GeneralConfig,
}

#[derive(Debug, Deserialize)]
struct GeneralConfig {
    name: String,
    #[serde(rename = "lastLaunchTime")]
    last_launch_time: Option<u64>,
    #[serde(rename = "totalTimePlayed")]
    total_time_played: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct MMCPack {
    components: Vec<Component>,
}

#[derive(Debug, Deserialize)]
struct Component {
    uid: String,
    version: Option<String>,
}

struct App {
    instances: Vec<Instance>,
    filtered_instances: Vec<Instance>,
    list_state: ListState,
    should_quit: bool,
    should_launch: bool,
    sort_mode: SortMode,
    search_mode: bool,
    search_query: String,
    details_mode: bool,
}

impl App {
    fn new() -> Result<Self> {
        let instances = load_instances()?;
        let filtered_instances = instances.clone();
        let mut list_state = ListState::default();
        if !instances.is_empty() {
            list_state.select(Some(0));
        }

        Ok(Self {
            instances,
            filtered_instances,
            list_state,
            should_quit: false,
            should_launch: false,
            sort_mode: SortMode::Name,
            search_mode: false,
            search_query: String::new(),
            details_mode: false,
        })
    }

    fn next(&mut self) {
        if self.filtered_instances.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.filtered_instances.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.filtered_instances.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_instances.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn launch_selected(&self) -> Result<()> {
        if let Some(selected) = self.list_state.selected() {
            if let Some(instance) = self.filtered_instances.get(selected) {
                launch_instance(&instance.name)?;
            }
        }
        Ok(())
    }

    fn open_folder_selected(&self) -> Result<()> {
        if let Some(selected) = self.list_state.selected() {
            if let Some(instance) = self.filtered_instances.get(selected) {
                Command::new("xdg-open")
                    .arg(&instance.path)
                    .spawn()?;
            }
        }
        Ok(())
    }

    fn cycle_sort(&mut self) {
        self.sort_mode = self.sort_mode.next();
        self.sort_instances();
        self.update_filter();
        // Reset selection to top
        if !self.filtered_instances.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    fn sort_instances(&mut self) {
        match self.sort_mode {
            SortMode::Name => {
                self.instances.sort_by(|a, b| a.name.cmp(&b.name));
            }
            SortMode::LastPlayed => {
                self.instances.sort_by(|a, b| {
                    b.last_played_ts.unwrap_or(0).cmp(&a.last_played_ts.unwrap_or(0))
                });
            }
            SortMode::Playtime => {
                self.instances.sort_by(|a, b| {
                    b.time_played_secs.unwrap_or(0).cmp(&a.time_played_secs.unwrap_or(0))
                });
            }
        }
    }

    fn update_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_instances = self.instances.clone();
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_instances = self.instances
                .iter()
                .filter(|instance| instance.name.to_lowercase().contains(&query))
                .cloned()
                .collect();
        }

        // Reset selection if needed
        if !self.filtered_instances.is_empty() && self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        } else if let Some(selected) = self.list_state.selected() {
            if selected >= self.filtered_instances.len() {
                self.list_state.select(Some(0));
            }
        }
    }

    fn enter_search_mode(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
        self.update_filter();
    }

    fn exit_search_mode(&mut self) {
        self.search_mode = false;
        self.search_query.clear();
        self.update_filter();
    }

    fn update_search_query(&mut self, c: char) {
        self.search_query.push(c);
        self.update_filter();
    }

    fn backspace_search(&mut self) {
        self.search_query.pop();
        self.update_filter();
    }

    fn toggle_details(&mut self) {
        self.details_mode = !self.details_mode;
    }
}

fn load_instances() -> Result<Vec<Instance>> {
    let home = std::env::var("HOME")?;
    let instances_path = Path::new(&home)
        .join(".var/app/org.prismlauncher.PrismLauncher/data/PrismLauncher/instances");

    if !instances_path.exists() {
        return Ok(Vec::new());
    }

    let mut instances = Vec::new();

    for entry in fs::read_dir(instances_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let config_path = path.join("instance.cfg");
            if config_path.exists() {
                if let Ok(config_str) = fs::read_to_string(&config_path) {
                    if let Ok(config) = serde_ini::from_str::<InstanceConfig>(&config_str) {
                        let last_played_ts = config.general.last_launch_time;
                        let last_played = last_played_ts.map(format_timestamp);

                        let time_played_secs = config.general.total_time_played;
                        let time_played = time_played_secs.map(format_duration);

                        // Try to get Minecraft version from mmc-pack.json
                        let mc_version = path.join("mmc-pack.json")
                            .exists()
                            .then(|| {
                                fs::read_to_string(path.join("mmc-pack.json"))
                                    .ok()
                                    .and_then(|content| serde_json::from_str::<MMCPack>(&content).ok())
                                    .and_then(|pack| {
                                        pack.components
                                            .iter()
                                            .find(|c| c.uid == "net.minecraft")
                                            .and_then(|c| c.version.clone())
                                    })
                            })
                            .flatten();

                        instances.push(Instance {
                            name: config.general.name,
                            path: path.clone(),
                            last_played,
                            last_played_ts,
                            time_played,
                            time_played_secs,
                            mc_version,
                        });
                    }
                }
            }
        }
    }

    instances.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(instances)
}

fn format_timestamp(timestamp_ms: u64) -> String {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    let duration = Duration::from_millis(timestamp_ms);
    let datetime = UNIX_EPOCH + duration;
    let elapsed = SystemTime::now().duration_since(datetime).ok();

    if let Some(elapsed) = elapsed {
        let days = elapsed.as_secs() / 86400;
        if days > 0 {
            return format!("{} days ago", days);
        }
        let hours = elapsed.as_secs() / 3600;
        if hours > 0 {
            return format!("{} hours ago", hours);
        }
        let minutes = elapsed.as_secs() / 60;
        if minutes > 0 {
            return format!("{} minutes ago", minutes);
        }
        "Just now".to_string()
    } else {
        "Recently".to_string()
    }
}

fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m", minutes)
    } else {
        format!("{}s", seconds)
    }
}

fn launch_instance(instance_name: &str) -> Result<()> {
    // Call the dedicated launch script
    let home = std::env::var("HOME")?;
    let script_path = format!("{}/scripts/launch-minecraft.sh", home);

    Command::new(&script_path)
        .arg(instance_name)
        .spawn()?;

    Ok(())
}

fn main() -> Result<()> {
    let mut terminal = setup_terminal()?;
    let mut app = App::new()?;
    let res = run_app(&mut terminal, &mut app);
    restore_terminal(&mut terminal)?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
        return Ok(());
    }

    // Launch instance AFTER terminal is restored
    if app.should_launch {
        app.launch_selected()?;
        // Give the script time to spawn the flatpak process
        thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if app.search_mode {
                    // In search mode
                    match key.code {
                        KeyCode::Esc => {
                            app.exit_search_mode();
                        }
                        KeyCode::Char(c) => {
                            app.update_search_query(c);
                        }
                        KeyCode::Backspace => {
                            app.backspace_search();
                        }
                        KeyCode::Enter => {
                            // Exit search and launch
                            app.exit_search_mode();
                            app.should_quit = true;
                            app.should_launch = true;
                        }
                        KeyCode::Down => {
                            app.next();
                        }
                        KeyCode::Up => {
                            app.previous();
                        }
                        _ => {}
                    }
                } else {
                    // Normal mode
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.should_quit = true;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.next();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.previous();
                        }
                        KeyCode::Enter => {
                            app.should_quit = true;
                            app.should_launch = true;
                        }
                        KeyCode::Char('o') => {
                            app.open_folder_selected()?;
                        }
                        KeyCode::Char('s') => {
                            app.cycle_sort();
                        }
                        KeyCode::Char('/') => {
                            app.enter_search_mode();
                        }
                        KeyCode::Char('i') => {
                            app.toggle_details();
                        }
                        _ => {}
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    if app.details_mode {
        // Details view: split horizontally
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.area());

        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(60),
            ])
            .split(main_chunks[1]);

        render_header(f, main_chunks[0]);
        render_instances(f, content_chunks[0], app);
        render_details(f, content_chunks[1], app);
        render_footer(f, main_chunks[2], app);
    } else if app.search_mode {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),  // Search bar
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.area());

        render_header(f, chunks[0]);
        render_search_bar(f, chunks[1], app);
        render_instances(f, chunks[2], app);
        render_footer(f, chunks[3], app);
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.area());

        render_header(f, chunks[0]);
        render_instances(f, chunks[1], app);
        render_footer(f, chunks[2], app);
    }
}

fn render_header(f: &mut Frame, area: Rect) {
    let title = Paragraph::new("⛏  Minecraft Instance Manager")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
        );
    f.render_widget(title, area);
}

fn render_search_bar(f: &mut Frame, area: Rect, app: &App) {
    let search_text = format!("Search: {}", app.search_query);
    let search_bar = Paragraph::new(search_text)
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Filter (ESC to exit) ")
        );
    f.render_widget(search_bar, area);
}

fn render_instances(f: &mut Frame, area: Rect, app: &mut App) {
    if app.filtered_instances.is_empty() {
        let message = if app.search_mode {
            Paragraph::new("No instances match your search")
        } else {
            Paragraph::new("No Minecraft instances found")
        }
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White))
                    .title(" Instances ")
            );
        f.render_widget(message, area);
        return;
    }

    let items: Vec<ListItem> = app
        .filtered_instances
        .iter()
        .map(|instance| {
            let is_running = is_instance_running(&instance.name);

            let mut title_spans = vec![
                Span::styled("▶ ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(&instance.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ];

            // Add version to title if available
            if let Some(ref version) = instance.mc_version {
                title_spans.push(Span::styled(
                    format!(" [{}]", version),
                    Style::default().fg(Color::Cyan)
                ));
            }

            // Add running indicator
            if is_running {
                title_spans.push(Span::styled(
                    " ● RUNNING",
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                ));
            }

            let mut lines = vec![Line::from(title_spans)];

            let mut info_parts = Vec::new();
            if let Some(ref time_played) = instance.time_played {
                info_parts.push(format!("Playtime: {}", time_played));
            }
            if let Some(ref last_played) = instance.last_played {
                info_parts.push(format!("Last played: {}", last_played));
            }

            if !info_parts.is_empty() {
                lines.push(Line::from(
                    Span::styled(
                        format!("  {}", info_parts.join(" • ")),
                        Style::default().fg(Color::DarkGray)
                    )
                ));
            }

            ListItem::new(lines).style(Style::default())
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Select Instance ")
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(50, 50, 80))
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let sort_text = format!(" Sort: {}  ", app.sort_mode.display());
    let help_text = vec![
        Span::styled("↑↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" Navigate  "),
        Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(" Launch  "),
        Span::styled("o", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" Open  "),
        Span::styled("s", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::raw(sort_text),
        Span::styled("/", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" Search  "),
        Span::styled("i", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
        Span::raw(" Details  "),
        Span::styled("q/Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw(" Quit"),
    ];

    let footer = Paragraph::new(Line::from(help_text))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White))
        );

    f.render_widget(footer, area);
}

fn render_details(f: &mut Frame, area: Rect, app: &App) {
    if let Some(selected) = app.list_state.selected() {
        if let Some(instance) = app.filtered_instances.get(selected) {
            let mut details_lines = vec![];

            details_lines.push(Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw(&instance.name),
            ]));

            if let Some(ref version) = instance.mc_version {
                details_lines.push(Line::from(vec![
                    Span::styled("Minecraft Version: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(version),
                ]));
            }

            details_lines.push(Line::from("")); // Blank line

            details_lines.push(Line::from(vec![
                Span::styled("Path: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]));
            details_lines.push(Line::from(
                Span::styled(instance.path.display().to_string(), Style::default().fg(Color::DarkGray))
            ));

            details_lines.push(Line::from("")); // Blank line

            if let Some(ref time_played) = instance.time_played {
                details_lines.push(Line::from(vec![
                    Span::styled("Total Playtime: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(time_played),
                ]));
            }

            if let Some(ref last_played) = instance.last_played {
                details_lines.push(Line::from(vec![
                    Span::styled("Last Played: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(last_played),
                ]));
            }

            // Count mods
            let mods_path = instance.path.join("mods");
            if mods_path.exists() {
                if let Ok(entries) = fs::read_dir(&mods_path) {
                    let mod_count = entries
                        .filter_map(|e| e.ok())
                        .filter(|e| {
                            e.path().extension()
                                .and_then(|ext| ext.to_str())
                                .map(|ext| ext == "jar")
                                .unwrap_or(false)
                        })
                        .count();

                    details_lines.push(Line::from("")); // Blank line
                    details_lines.push(Line::from(vec![
                        Span::styled("Mods: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                        Span::raw(format!("{} installed", mod_count)),
                    ]));
                }
            }

            let details = Paragraph::new(details_lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Blue))
                        .title(" Instance Details (i to close) ")
                )
                .wrap(ratatui::widgets::Wrap { trim: false });

            f.render_widget(details, area);
            return;
        }
    }

    // No instance selected
    let message = Paragraph::new("No instance selected")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
                .title(" Instance Details ")
        );
    f.render_widget(message, area);
}
