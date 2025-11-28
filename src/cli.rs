use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "nvproton command-line interface",
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Detect(DetectArgs),
    Profile(ProfileArgs),
    Config(ConfigArgs),
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
