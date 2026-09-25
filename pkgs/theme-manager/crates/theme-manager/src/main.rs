mod doctor;
mod health;
mod registry;
mod runtime;
mod tui;
mod watch;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use registry::Registry;
use runtime::Runtime;
use serde::Serialize;
use std::{
    io::{self, Write},
    path::{Path, PathBuf},
};
use theme_core::{Config, ExtensionsConfig};
use tracing_subscriber::EnvFilter;

/// Write a complete line without panicking when a downstream pipeline closes.
macro_rules! outputln {
    ($($argument:tt)*) => {{
        if let Err(error) = writeln!(io::stdout().lock(), $($argument)*) {
            if error.kind() == io::ErrorKind::BrokenPipe {
                return Ok(());
            }
            return Err(error.into());
        }
    }};
}

#[derive(Parser)]
#[command(
    name = "theme-manager",
    version,
    about = "Modular desktop structure/theme manager"
)]
struct Cli {
    /// Configuration file. Defaults to $XDG_CONFIG_HOME/theme-manager/config.toml.
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List available structure profiles.
    List {
        #[arg(long)]
        json: bool,
    },

    /// Apply a structure profile and make it active.
    Apply {
        profile: String,
        #[arg(long)]
        dry_run: bool,
        /// Explicitly replace modified or unmanaged files at configured output paths.
        #[arg(long)]
        force: bool,
        #[arg(long)]
        json: bool,
    },

    /// Show the fully resolved structure after dynamic provider values are applied.
    Resolve {
        profile: Option<String>,
        #[arg(long)]
        json: bool,
    },

    /// Show current provider, profile and targets.
    Status {
        #[arg(long)]
        json: bool,
    },

    /// Validate one profile, or every profile when no name is supplied.
    Check { profile: Option<String> },

    /// Manage desktop structure profiles.
    Structure {
        #[command(subcommand)]
        action: StructureAction,
    },

    /// Diagnose configuration, profiles, outputs, plugins, and watcher health.
    Doctor {
        #[arg(long)]
        json: bool,
    },

    /// Watch profiles, provider output, state and config; reapply live on changes.
    Watch,

    /// Show available adapters or control configured target instances.
    Components {
        #[command(subcommand)]
        action: Option<ComponentAction>,
        #[arg(long, global = true)]
        json: bool,
    },

    /// Inspect and validate sandboxed Luau extensions.
    Plugins {
        #[command(subcommand)]
        action: PluginAction,
        #[arg(long, global = true)]
        json: bool,
    },

    /// Open the interactive terminal interface.
    Tui {
        /// Disable Vim-style navigation for this run.
        #[arg(long)]
        no_vim: bool,
    },
}

#[derive(Subcommand)]
enum StructureAction {
    /// List available structure profiles.
    List {
        #[arg(long)]
        json: bool,
    },
    /// Apply a structure profile and make it active.
    Apply {
        profile: String,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        force: bool,
        #[arg(long)]
        json: bool,
    },
    /// Show the fully resolved structure.
    Resolve {
        profile: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Validate one structure, or every structure when omitted.
    Check { profile: Option<String> },
}

#[derive(Subcommand)]
enum ComponentAction {
    /// Show configured targets and their current runtime state.
    List,
    /// Enable a configured target and regenerate its output.
    Enable { target: String },
    /// Disable a configured target and remove its generated output.
    Disable { target: String },
}

#[derive(Subcommand)]
enum PluginAction {
    /// List discovered extensions and any discovery diagnostics.
    List,
    /// Compile and validate discovered extension manifests and entry scripts.
    Check { id: Option<String> },
}

#[derive(Serialize)]
struct Status<'a> {
    active_profile: Option<String>,
    provider: &'a str,
    targets: Vec<&'a str>,
}

#[derive(Serialize)]
struct Components {
    providers: Vec<String>,
    targets: Vec<String>,
}

#[derive(Serialize)]
struct Plugins<'a> {
    extensions: &'a [theme_extension_luau::ExtensionDescriptor],
    diagnostics: &'a [String],
}

fn setup_logging() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("theme_manager=info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

fn print_json<T: Serialize>(value: &T) -> Result<()> {
    outputln!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn inspection_registry(config: Option<&Path>) -> Result<Registry> {
    if let Some(config) = config {
        let (config, _) = Config::load(Some(config))?;
        return Registry::load(&config);
    }

    match theme_core::paths::default_config_path() {
        Ok(path) if path.is_file() => {
            let (config, _) = Config::load(Some(&path))?;
            Registry::load(&config)
        }
        Ok(_) | Err(_) => Registry::load_extensions(&ExtensionsConfig::default()),
    }
}

fn main() -> Result<()> {
    setup_logging();
    let cli = Cli::parse();

    if let Command::Components { action: None, json } = &cli.command {
        let registry = inspection_registry(cli.config.as_deref())?;
        let components = Components {
            providers: registry.provider_ids().cloned().collect(),
            targets: registry.target_ids().cloned().collect(),
        };
        if *json {
            return print_json(&components);
        }
        outputln!("Providers: {}", components.providers.join(", "));
        outputln!("Targets:   {}", components.targets.join(", "));
        return Ok(());
    }

    if let Command::Plugins { action, json } = &cli.command {
        let registry = inspection_registry(cli.config.as_deref())?;
        let plugins = Plugins {
            extensions: registry.extensions(),
            diagnostics: registry.diagnostics(),
        };
        match action {
            PluginAction::List => {
                if *json {
                    return print_json(&plugins);
                }
                if plugins.extensions.is_empty() {
                    outputln!("No Luau extensions discovered.");
                }
                for extension in plugins.extensions {
                    outputln!(
                        "{:<24} {:<8} {}",
                        extension.adapter_id,
                        format!("{:?}", extension.kind).to_lowercase(),
                        extension.entry_path.display()
                    );
                }
                for diagnostic in plugins.diagnostics {
                    eprintln!("error: {diagnostic}");
                }
                return Ok(());
            }
            PluginAction::Check { id } => {
                if !plugins.diagnostics.is_empty() {
                    anyhow::bail!(
                        "extension validation failed:\n{}",
                        plugins.diagnostics.join("\n")
                    );
                }
                let selected = plugins
                    .extensions
                    .iter()
                    .filter(|extension| {
                        id.as_ref().is_none_or(|id| {
                            extension.extension_id == id.strip_prefix("luau:").unwrap_or(id)
                        })
                    })
                    .collect::<Vec<_>>();
                if let Some(id) = id
                    && selected.is_empty()
                {
                    anyhow::bail!("unknown Luau extension '{id}'");
                }
                for extension in &selected {
                    theme_extension_luau::validate_extension(extension).with_context(|| {
                        format!("extension '{}' is invalid", extension.adapter_id)
                    })?;
                }
                if *json {
                    return print_json(&plugins);
                }
                outputln!("ok: {} Luau extension(s)", selected.len());
                return Ok(());
            }
        }
    }

    if matches!(&cli.command, Command::Watch) {
        return watch::run(cli.config);
    }

    if let Command::Doctor { json } = &cli.command {
        return doctor::run(cli.config.as_deref(), *json);
    }

    if let Command::Tui { no_vim } = &cli.command {
        return tui::run(cli.config, *no_vim);
    }

    let runtime =
        Runtime::load(cli.config.as_deref()).context("failed to initialize theme-manager")?;

    match cli.command {
        Command::List { json } => {
            list_structures(&runtime, json)?;
        }

        Command::Apply {
            profile,
            dry_run,
            force,
            json,
        } => {
            apply_structure(&runtime, &profile, dry_run, force, json)?;
        }

        Command::Resolve { profile, json } => {
            resolve_structure(&runtime, profile, json)?;
        }

        Command::Status { json } => {
            let status = Status {
                active_profile: runtime.engine.active_profile()?,
                provider: runtime.engine.provider_id(),
                targets: runtime.engine.target_ids(),
            };
            if json {
                print_json(&status)?;
            } else {
                outputln!(
                    "Profile:  {}",
                    status.active_profile.as_deref().unwrap_or("<none>")
                );
                outputln!("Provider: {}", status.provider);
                outputln!("Targets:  {}", status.targets.join(", "));
            }
        }

        Command::Check { profile } => {
            check_structures(&runtime, profile)?;
        }

        Command::Structure { action } => match action {
            StructureAction::List { json } => list_structures(&runtime, json)?,
            StructureAction::Apply {
                profile,
                dry_run,
                force,
                json,
            } => apply_structure(&runtime, &profile, dry_run, force, json)?,
            StructureAction::Resolve { profile, json } => {
                resolve_structure(&runtime, profile, json)?
            }
            StructureAction::Check { profile } => check_structures(&runtime, profile)?,
        },

        Command::Components {
            action: Some(action),
            json,
        } => match action {
            ComponentAction::List => {
                let targets = runtime.target_statuses();
                if json {
                    print_json(&targets)?;
                } else {
                    outputln!("Provider: {}", runtime.engine.provider_id());
                    for target in targets {
                        let state = if target.enabled {
                            "enabled"
                        } else if !target.configured {
                            "not configured"
                        } else if !target.toggleable {
                            "disabled in config"
                        } else {
                            "disabled"
                        };
                        let label = if target.target == target.adapter {
                            target.target
                        } else {
                            format!("{} ({})", target.target, target.adapter)
                        };
                        outputln!("  {label:<28} {state}");
                    }
                }
            }
            ComponentAction::Enable { target } => {
                print_component_change(&runtime, &target, true, json)?;
            }
            ComponentAction::Disable { target } => {
                print_component_change(&runtime, &target, false, json)?;
            }
        },

        Command::Watch
        | Command::Doctor { .. }
        | Command::Tui { .. }
        | Command::Plugins { .. }
        | Command::Components { action: None, .. } => {
            unreachable!()
        }
    }

    Ok(())
}

fn list_structures(runtime: &Runtime, json: bool) -> Result<()> {
    let profiles = runtime.engine.list_profiles()?;
    if json {
        return print_json(&profiles);
    }
    for profile in profiles {
        outputln!(
            "{} {}",
            if profile.active { "*" } else { " " },
            profile.name
        );
    }
    Ok(())
}

fn apply_structure(
    runtime: &Runtime,
    profile: &str,
    dry_run: bool,
    force: bool,
    json: bool,
) -> Result<()> {
    let report = runtime.engine.apply_with_options(profile, dry_run, force)?;
    if json {
        return print_json(&report);
    }
    outputln!(
        "{} profile '{}'",
        if dry_run { "Resolved" } else { "Applied" },
        profile
    );
    for target in &report.targets {
        outputln!(
            "  {}: {} changed, {} unchanged",
            target.target,
            target.changed.len(),
            target.unchanged.len()
        );
        for path in &target.changed {
            outputln!("    -> {}", path.display());
        }
        for path in &target.forced {
            outputln!("    !! forcibly replaced {}", path.display());
        }
    }
    Ok(())
}

fn resolve_structure(runtime: &Runtime, profile: Option<String>, json: bool) -> Result<()> {
    let profile = match profile {
        Some(profile) => profile,
        None => runtime
            .engine
            .active_profile()?
            .context("no active/default profile")?,
    };
    let resolved = runtime.engine.resolve_profile(&profile)?;
    if json {
        print_json(&resolved)
    } else {
        outputln!("{}", toml::to_string_pretty(&resolved)?);
        Ok(())
    }
}

fn check_structures(runtime: &Runtime, profile: Option<String>) -> Result<()> {
    if let Some(profile) = profile {
        runtime.engine.validate_profile(&profile)?;
        outputln!("ok: {profile}");
        return Ok(());
    }
    for profile in runtime.engine.list_profiles()? {
        runtime
            .engine
            .validate_profile(&profile.name)
            .with_context(|| format!("profile '{}' is invalid", profile.name))?;
        outputln!("ok: {}", profile.name);
    }
    Ok(())
}

fn print_component_change(
    runtime: &Runtime,
    target: &str,
    enabled: bool,
    json: bool,
) -> Result<()> {
    let report = runtime.set_target_enabled(target, enabled)?;
    if json {
        return print_json(&report);
    }

    outputln!(
        "{} target '{}'",
        if enabled { "Enabled" } else { "Disabled" },
        target
    );
    for path in report.changed {
        outputln!("  generated {}", path.display());
    }
    for path in report.removed {
        outputln!("  removed   {}", path.display());
    }
    Ok(())
}
