use anyhow::Result;

use crate::cli::{
    GamesArgs, GamesCommand, GamesInfoArgs, GamesListArgs, GamesScanArgs, GamesSetProfileArgs,
    GamesShowArgs, OutputFormat,
};
use crate::config::{ConfigManager, NvConfig};
use crate::detection::{self, DetectionContext, GameDatabase, GameSource};

/// Handle the `games` command
pub fn handle_games(args: GamesArgs, manager: &ConfigManager, config: &mut NvConfig) -> Result<()> {
    match args.command {
        GamesCommand::List(list_args) => handle_list(list_args, manager, config),
        GamesCommand::Show(show_args) => handle_show(show_args, manager, config),
        GamesCommand::Scan(scan_args) => handle_scan(scan_args, manager, config),
        GamesCommand::SetProfile(set_args) => handle_set_profile(set_args, manager, config),
        GamesCommand::Info(info_args) => handle_info(info_args, manager, config),
    }
}

fn handle_list(args: GamesListArgs, manager: &ConfigManager, _config: &NvConfig) -> Result<()> {
    let db = GameDatabase::load_or_default(manager.paths())?;
    let games: Vec<_> = db
        .games()
        .filter(|g| {
            if let Some(ref source) = args.source {
                matches!(
                    (&g.source, source.as_str()),
                    (GameSource::Steam, "steam")
                        | (GameSource::Heroic, "heroic")
                        | (GameSource::Lutris, "lutris")
                )
            } else {
                true
            }
        })
        .collect();

    if games.is_empty() {
        println!("No games found. Run 'nvproton games scan' to detect games.");
        return Ok(());
    }

    match args.format {
        OutputFormat::Text => {
            println!("{:<12} {:<10} Name", "ID", "Source");
            println!("{}", "-".repeat(60));
            for game in &games {
                println!("{:<12} {:<10} {}", game.id, game.source, game.name);
            }
            println!("\n{} games found", games.len());
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&games)?);
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(&games)?);
        }
    }

    Ok(())
}

fn handle_show(args: GamesShowArgs, manager: &ConfigManager, _config: &NvConfig) -> Result<()> {
    let db = GameDatabase::load_or_default(manager.paths())?;

    if let Some(game) = db.get(&args.game_id) {
        println!("Name:        {}", game.name);
        println!("ID:          {}", game.id);
        println!("Source:      {}", game.source);
        println!("Install Dir: {:?}", game.install_dir);
        if let Some(exe) = &game.executable {
            println!("Executable:  {:?}", exe);
        }
        if let Some(fp) = &game.fingerprint {
            println!("Fingerprint: {}", fp);
        }
        if !game.metadata.is_empty() {
            println!("Metadata:");
            for (key, value) in &game.metadata {
                println!("  {}: {}", key, value);
            }
        }
    } else {
        anyhow::bail!("Game '{}' not found in database", args.game_id);
    }

    Ok(())
}

fn handle_scan(args: GamesScanArgs, manager: &ConfigManager, config: &mut NvConfig) -> Result<()> {
    let ctx = DetectionContext::new(config, manager);
    let mut all_games = Vec::new();

    println!("Scanning for games...\n");

    // Steam
    print!("  Steam: ");
    match detection::steam::SteamDetector::new().detect(&ctx, args.fingerprint) {
        Ok(games) => {
            println!("{} games found", games.len());
            all_games.extend(games);
        }
        Err(e) => println!("error - {}", e),
    }

    // Heroic
    print!("  Heroic: ");
    match detection::heroic::HeroicDetector::new().detect(&ctx, args.fingerprint) {
        Ok(games) => {
            println!("{} games found", games.len());
            all_games.extend(games);
        }
        Err(e) => println!("error - {}", e),
    }

    // Lutris
    print!("  Lutris: ");
    match detection::lutris::LutrisDetector::new().detect(&ctx, args.fingerprint) {
        Ok(games) => {
            println!("{} games found", games.len());
            all_games.extend(games);
        }
        Err(e) => println!("error - {}", e),
    }

    // Update database
    let mut db = GameDatabase::load_or_default(manager.paths())?;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    // Clean out old excluded entries (Proton, Runtime, etc.)
    let cleaned = db.cleanup_excluded();
    if cleaned > 0 {
        println!("  Cleaned: {} excluded entries removed", cleaned);
    }

    db.merge_detected(&all_games, timestamp);
    db.save(manager.paths())?;

    println!("\nTotal: {} games added to database", all_games.len());
    println!("Use 'nvproton games list' to see all games");

    Ok(())
}

fn handle_set_profile(
    args: GamesSetProfileArgs,
    manager: &ConfigManager,
    _config: &NvConfig,
) -> Result<()> {
    let mut db = GameDatabase::load_or_default(manager.paths())?;

    if db.get(&args.game_id).is_none() {
        anyhow::bail!("Game '{}' not found in database", args.game_id);
    }

    // Verify profile exists
    let profile_manager = crate::profile::ProfileManager::new(manager.paths().profiles_dir.clone());
    if !profile_manager.exists(&args.profile) {
        anyhow::bail!(
            "Profile '{}' not found. Use 'nvproton profile list' to see available profiles.",
            args.profile
        );
    }

    db.set_game_profile(&args.game_id, &args.profile);
    db.save(manager.paths())?;

    println!(
        "Profile '{}' assigned to game '{}'",
        args.profile, args.game_id
    );
    Ok(())
}

fn handle_info(args: GamesInfoArgs, manager: &ConfigManager, _config: &NvConfig) -> Result<()> {
    let db = GameDatabase::load_or_default(manager.paths())?;

    if let Some(game) = db.get(&args.game_id) {
        println!("Game: {} ({})", game.name, game.id);
        println!();

        // Show recommended launch command
        if args.command {
            println!("Launch Command:");
            match game.source {
                GameSource::Steam => {
                    println!("  nvproton run {} --reflex --vrr", game.id);
                    println!();
                    println!("Or with Steam directly:");
                    println!("  steam -applaunch {}", game.id);
                }
                GameSource::Heroic => {
                    println!("  nvproton run {} --reflex", game.id);
                    println!();
                    println!("Or with Heroic directly:");
                    println!("  heroic --launch {}", game.id);
                }
                GameSource::Lutris => {
                    println!("  nvproton run {}", game.id);
                    println!();
                    println!("Or with Lutris directly:");
                    println!("  lutris lutris:rungame/{}", game.id);
                }
                GameSource::Unknown => {
                    if let Some(exe) = &game.executable {
                        println!("  {:?}", exe);
                    } else {
                        println!("  (no executable found)");
                    }
                }
            }
        } else {
            // Show quick info
            println!("Source: {}", game.source);
            println!("Install: {:?}", game.install_dir);

            // Show associated profile if any
            if let Some(profile) = db.get_game_profile(&args.game_id) {
                println!("Profile: {}", profile);
            }

            println!();
            println!("Use --command to see launch options");
        }
    } else {
        anyhow::bail!("Game '{}' not found in database", args.game_id);
    }

    Ok(())
}
