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

#[derive(Debug, Clone)]
struct Instance {
    name: String,
    path: PathBuf,
    last_played: Option<String>,
    time_played: Option<String>,
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

struct App {
    instances: Vec<Instance>,
    list_state: ListState,
    should_quit: bool,
    should_launch: bool,
}

impl App {
    fn new() -> Result<Self> {
        let instances = load_instances()?;
        let mut list_state = ListState::default();
        if !instances.is_empty() {
            list_state.select(Some(0));
        }

        Ok(Self {
            instances,
            list_state,
            should_quit: false,
            should_launch: false,
        })
    }

    fn next(&mut self) {
        if self.instances.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.instances.len() - 1 {
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
        if self.instances.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.instances.len() - 1
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
            if let Some(instance) = self.instances.get(selected) {
                launch_instance(&instance.name)?;
            }
        }
        Ok(())
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
                        let last_played = config.general.last_launch_time.map(|ts| {
                            format_timestamp(ts)
                        });

                        let time_played = config.general.total_time_played.map(|seconds| {
                            format_duration(seconds)
                        });

                        instances.push(Instance {
                            name: config.general.name,
                            path: path.clone(),
                            last_played,
                            time_played,
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
                    _ => {}
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
    render_footer(f, chunks[2]);
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

fn render_instances(f: &mut Frame, area: Rect, app: &mut App) {
    if app.instances.is_empty() {
        let message = Paragraph::new("No Minecraft instances found")
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
        .instances
        .iter()
        .map(|instance| {
            let mut lines = vec![Line::from(vec![
                Span::styled("▶ ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(&instance.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ])];

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

fn render_footer(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Span::styled("↑↓", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" Navigate  "),
        Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw(" Launch  "),
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
