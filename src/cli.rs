use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "nvproton - NVIDIA-optimized Proton game launcher",
    propagate_version = true,
    after_help = "Examples:\n  nvproton run 1245620              # Run Elden Ring by Steam AppID\n  nvproton run --name \"Elden Ring\"  # Run by game name\n  nvproton prepare 1245620          # Pre-warm shaders before launch\n  nvproton games list               # List detected games"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Run a game with NVIDIA optimizations
    Run(RunArgs),
    /// Prepare a game (shader pre-warming, profile setup)
    Prepare(PrepareArgs),
    /// Manage detected games
    Games(GamesArgs),
    /// Steam integration (launch options, Proton, shortcuts)
    Steam(SteamArgs),
    /// Detect games from various sources
    Detect(DetectArgs),
    /// Manage game profiles
    Profile(ProfileArgs),
    /// Manage nvproton configuration
    Config(ConfigArgs),
}

#[derive(Debug, Args)]
pub struct RunArgs {
    /// Steam AppID or game identifier
    #[arg(value_name = "GAME_ID")]
    pub game_id: Option<String>,

    /// Run game by name (fuzzy match)
    #[arg(long)]
    pub name: Option<String>,

    /// Profile to apply
    #[arg(short, long)]
    pub profile: Option<String>,

    /// Enable Reflex low-latency mode
    #[arg(long)]
    pub reflex: bool,

    /// Target frame rate (0 = unlimited)
    #[arg(long, default_value = "0")]
    pub fps: u32,

    /// Enable VRR (G-Sync/FreeSync)
    #[arg(long)]
    pub vrr: bool,

    /// Skip shader pre-warming
    #[arg(long)]
    pub no_prewarm: bool,

    /// Dry run - show what would be done without launching
    #[arg(long)]
    pub dry_run: bool,

    /// Additional arguments to pass to the game
    #[arg(last = true)]
    pub game_args: Vec<String>,
}

#[derive(Debug, Args)]
pub struct PrepareArgs {
    /// Steam AppID or game identifier
    #[arg(value_name = "GAME_ID")]
    pub game_id: Option<String>,

    /// Prepare game by name (fuzzy match)
    #[arg(long)]
    pub name: Option<String>,

    /// Profile to apply
    #[arg(short, long)]
    pub profile: Option<String>,

    /// Force shader recompilation
    #[arg(long)]
    pub force: bool,

    /// Show progress during shader compilation
    #[arg(long, default_value = "true")]
    pub progress: bool,
}

#[derive(Debug, Args)]
pub struct GamesArgs {
    #[command(subcommand)]
    pub command: GamesCommand,
}

#[derive(Debug, Subcommand)]
pub enum GamesCommand {
    /// List all detected games
    List(GamesListArgs),
    /// Show details for a specific game
    Show(GamesShowArgs),
    /// Scan for new games
    Scan(GamesScanArgs),
    /// Assign a profile to a game
    SetProfile(GamesSetProfileArgs),
    /// Show game launch command
    Info(GamesInfoArgs),
}

#[derive(Debug, Args)]
pub struct GamesListArgs {
    /// Filter by source (steam, heroic, lutris)
    #[arg(long)]
    pub source: Option<String>,

    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,
}

#[derive(Debug, Args)]
pub struct GamesShowArgs {
    /// Steam AppID or game identifier
    pub game_id: String,
}

#[derive(Debug, Args)]
pub struct GamesScanArgs {
    /// Rescan all sources
    #[arg(long)]
    pub all: bool,

    /// Generate fingerprints for executables
    #[arg(long)]
    pub fingerprint: bool,
}

#[derive(Debug, Args)]
pub struct GamesSetProfileArgs {
    /// Steam AppID or game identifier
    pub game_id: String,

    /// Profile name to assign
    pub profile: String,
}

#[derive(Debug, Args)]
pub struct GamesInfoArgs {
    /// Steam AppID or game identifier
    pub game_id: String,

    /// Show full launch command
    #[arg(long)]
    pub command: bool,
}

#[derive(Debug, Args)]
pub struct DetectArgs {
    #[command(subcommand)]
    pub command: DetectCommand,
}

#[derive(Debug, Subcommand)]
pub enum DetectCommand {
    Steam(DetectSourceArgs),
    Heroic(DetectSourceArgs),
    Lutris(DetectSourceArgs),
    All(DetectAllArgs),
}

#[derive(Debug, Args)]
pub struct DetectSourceArgs {
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,
    #[arg(long)]
    pub update_db: bool,
    #[arg(long)]
    pub fingerprint: bool,
}

#[derive(Debug, Args)]
pub struct DetectAllArgs {
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,
    #[arg(long)]
    pub update_db: bool,
    #[arg(long)]
    pub fingerprint: bool,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
    Yaml,
}

#[derive(Debug, Args)]
pub struct ProfileArgs {
    #[command(subcommand)]
    pub command: ProfileCommand,
}

#[derive(Debug, Subcommand)]
pub enum ProfileCommand {
    List,
    Show(ProfileNameArgs),
    Create(ProfileCreateArgs),
    Set(ProfileSetArgs),
    Import(ProfileImportArgs),
    Export(ProfileExportArgs),
}

#[derive(Debug, Args)]
pub struct ProfileNameArgs {
    pub name: String,
}

#[derive(Debug, Args)]
pub struct ProfileCreateArgs {
    pub name: String,
    #[arg(long)]
    pub base: Option<String>,
    #[arg(long = "set", value_parser = parse_kv_pair)]
    pub values: Vec<(String, String)>,
}

#[derive(Debug, Args)]
pub struct ProfileSetArgs {
    pub name: String,
    #[arg(long = "set", value_parser = parse_kv_pair)]
    pub values: Vec<(String, String)>,
}

#[derive(Debug, Args)]
pub struct ProfileImportArgs {
    pub path: String,
    #[arg(long)]
    pub name: Option<String>,
}

#[derive(Debug, Args)]
pub struct ProfileExportArgs {
    pub name: String,
    #[arg(long, value_enum, default_value_t = OutputFormat::Yaml)]
    pub format: OutputFormat,
    #[arg(long)]
    pub path: Option<String>,
}

#[derive(Debug, Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    Show,
    Paths,
    Reset,
}

fn parse_kv_pair(s: &str) -> Result<(String, String), String> {
    let (key, value) = s
        .split_once('=')
        .ok_or_else(|| "expected KEY=VALUE format".to_string())?;
    let key = key.trim();
    let value = value.trim();
    if key.is_empty() {
        return Err("key cannot be empty".into());
    }
    Ok((key.to_string(), value.to_string()))
}

// ============================================================================
// Steam Integration Commands
// ============================================================================

#[derive(Debug, Args)]
pub struct SteamArgs {
    #[command(subcommand)]
    pub command: SteamCommand,
}

#[derive(Debug, Subcommand)]
pub enum SteamCommand {
    /// Generate optimized launch options for a game
    LaunchOptions(LaunchOptionsArgs),
    /// Manage Proton versions
    Proton(ProtonArgs),
    /// Manage non-Steam shortcuts
    Shortcut(ShortcutArgs),
}

#[derive(Debug, Args)]
pub struct LaunchOptionsArgs {
    /// Steam AppID
    pub game_id: String,

    /// Use nvproton as launch wrapper
    #[arg(long, default_value = "true")]
    pub use_nvproton: bool,

    /// Enable Reflex low-latency mode
    #[arg(long)]
    pub reflex: bool,

    /// Enable VRR (G-Sync/FreeSync)
    #[arg(long)]
    pub vrr: bool,

    /// Target frame rate (0 = unlimited)
    #[arg(long, default_value = "0")]
    pub fps: u32,

    /// Use dedicated shader cache path
    #[arg(long)]
    pub shader_cache: bool,

    /// Enable MangoHud overlay
    #[arg(long)]
    pub mangohud: bool,

    /// Enable Feral Gamemode
    #[arg(long)]
    pub gamemode: bool,

    /// Additional environment variables (KEY=VALUE)
    #[arg(long = "env", value_parser = parse_kv_pair)]
    pub env: Vec<(String, String)>,

    /// Output in copy-paste format for Steam
    #[arg(long)]
    pub copy_format: bool,
}

#[derive(Debug, Args)]
pub struct ProtonArgs {
    #[command(subcommand)]
    pub command: ProtonCommand,
}

#[derive(Debug, Subcommand)]
pub enum ProtonCommand {
    /// List installed Proton versions
    List,
    /// Show recommended Proton versions for NVIDIA
    Recommended,
    /// Set default Proton version (shows instructions)
    SetDefault {
        /// Proton version name
        version: String,
    },
}

#[derive(Debug, Args)]
pub struct ShortcutArgs {
    #[command(subcommand)]
    pub command: ShortcutCommand,
}

#[derive(Debug, Subcommand)]
pub enum ShortcutCommand {
    /// Create a non-Steam shortcut
    Create {
        /// Shortcut name
        name: String,
        /// Executable path
        exe: String,
        /// Start directory
        #[arg(long)]
        start_dir: Option<String>,
        /// Icon path
        #[arg(long)]
        icon: Option<String>,
        /// Launch options
        #[arg(long)]
        launch_options: Option<String>,
    },
    /// List existing non-Steam shortcuts
    List,
    /// Generate optimized settings for a shortcut
    Optimize {
        /// Steam AppID or shortcut ID
        appid: String,
        /// Profile to apply
        #[arg(long)]
        profile: Option<String>,
    },
}
