mod adapters;
mod config;
mod health;
mod providers;
mod runtime;
mod watch;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use runtime::Runtime;
use serde::Serialize;
use std::{
    io::{self, Write},
    path::PathBuf,
};
use tracing_subscriber::EnvFilter;

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
    about = "Portable color and structure theme manager"
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
    /// Inspect configured Theme Bundles.
    Theme {
        #[command(subcommand)]
        action: ThemeAction,
    },
    /// Inspect portable Structure Profiles.
    Structure {
        #[command(subcommand)]
        action: StructureAction,
    },
    /// Inspect named Color Palettes.
    Palette {
        #[command(subcommand)]
        action: PaletteAction,
    },
    /// Inspect one explicit Appearance Library.
    Library {
        #[command(subcommand)]
        action: LibraryAction,
    },
    /// Show desired/effective selection and its Capability Plan.
    Status {
        #[arg(long)]
        json: bool,
    },
    /// Apply a complete Theme Bundle.
    Apply {
        theme: String,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
    },
    /// Validate one Theme Bundle, or all bundles when omitted.
    Check {
        theme: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Watch active inputs and reapply the active Theme Bundle.
    Watch,
}

#[derive(Subcommand)]
enum ThemeAction {
    List {
        #[arg(long)]
        json: bool,
    },
    Show {
        theme: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum StructureAction {
    List {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum PaletteAction {
    List {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum LibraryAction {
    List {
        library: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Serialize)]
struct ThemeDetails<'a> {
    name: &'a str,
    bundle: &'a theme_core::ThemeBundle,
    capability_plan: theme_core::CapabilityPlan,
    library_selections: Vec<theme_core::LibrarySelectionStatus>,
}

#[derive(Serialize)]
struct StatusReport {
    active_theme: Option<String>,
    capability_plan: Option<theme_core::CapabilityPlan>,
    library_selections: Vec<theme_core::LibrarySelectionStatus>,
    watcher: Option<health::WatchHealth>,
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

fn main() -> Result<()> {
    setup_logging();
    let cli = Cli::parse();
    if matches!(cli.command, Command::Watch) {
        return watch::run(cli.config);
    }
    let runtime =
        Runtime::load(cli.config.as_deref()).context("failed to initialize theme-manager")?;

    match cli.command {
        Command::Theme {
            action: ThemeAction::List { json },
        } => {
            let themes = runtime.themes()?;
            if json {
                print_json(&themes)?;
            } else {
                for theme in themes {
                    outputln!("{} {}", if theme.active { "*" } else { " " }, theme.name);
                }
            }
        }
        Command::Theme {
            action: ThemeAction::Show { theme, json },
        } => {
            let bundle = runtime
                .config
                .theme_bundles
                .get(&theme)
                .with_context(|| format!("unknown theme bundle '{theme}'"))?;
            let (capability_plan, library_selections) = runtime.plan(&theme)?;
            let details = ThemeDetails {
                name: &theme,
                bundle,
                capability_plan,
                library_selections,
            };
            if json {
                print_json(&details)?;
            } else {
                outputln!("{}", toml::to_string_pretty(bundle)?);
                print_plan(&details.capability_plan)?;
                print_library_selections(&details.library_selections)?;
            }
        }
        Command::Structure {
            action: StructureAction::List { json },
        } => {
            let structures = runtime.structures()?;
            if json {
                print_json(&structures)?;
            } else {
                for structure in structures {
                    outputln!(
                        "{} {}",
                        if structure.active { "*" } else { " " },
                        structure.name
                    );
                }
            }
        }
        Command::Palette {
            action: PaletteAction::List { json },
        } => {
            let palettes = runtime.palettes()?;
            if json {
                print_json(&palettes)?;
            } else {
                for palette in palettes {
                    outputln!(
                        "{} {:<20} provider={} source={:?}",
                        if palette.active { "*" } else { " " },
                        palette.name,
                        palette.provider,
                        palette.source
                    );
                }
            }
        }
        Command::Library {
            action: LibraryAction::List { library, json },
        } => {
            let selected = runtime
                .inventory
                .libraries
                .get(&library)
                .with_context(|| format!("unknown appearance library '{library}'"))?;
            if json {
                print_json(selected)?;
            } else {
                outputln!(
                    "Library: {library} ({:?}, {})",
                    selected.kind,
                    selected.source
                );
                for (name, item) in &selected.items {
                    outputln!(
                        "  {:<24} installed={} effective={} cadence={:?}",
                        name,
                        item.installed,
                        item.effective,
                        item.selection_cadence
                    );
                }
            }
        }
        Command::Status { json } => {
            let active_theme = runtime.active_theme()?;
            let (capability_plan, library_selections) = match &active_theme {
                Some(theme) => {
                    let (plan, selections) = runtime.plan(theme)?;
                    (Some(plan), selections)
                }
                None => (None, Vec::new()),
            };
            let health_path = health::path_for_state(&runtime.config.state_file()?);
            let watcher = health::read(&health_path)?;
            let status = StatusReport {
                active_theme,
                capability_plan,
                library_selections,
                watcher,
            };
            if json {
                print_json(&status)?;
            } else {
                outputln!(
                    "Theme: {}",
                    status.active_theme.as_deref().unwrap_or("<none>")
                );
                if let Some(plan) = &status.capability_plan {
                    print_plan(plan)?;
                }
                print_library_selections(&status.library_selections)?;
            }
        }
        Command::Apply {
            theme,
            dry_run,
            json,
        } => {
            let report = runtime.apply(&theme, dry_run)?;
            if json {
                print_json(&report)?;
            } else {
                outputln!(
                    "{} theme '{}'",
                    if dry_run { "Validated" } else { "Applied" },
                    theme
                );
                print_plan(&report.capability_plan)?;
                print_library_selections(&report.library_selections)?;
                for wire in report.wires {
                    outputln!(
                        "  {}: {} changed, {} unchanged{}",
                        wire.wire,
                        wire.changed.len(),
                        wire.unchanged.len(),
                        if wire.delegated { ", delegated" } else { "" }
                    );
                }
            }
        }
        Command::Check { theme, json } => {
            let names = match theme {
                Some(theme) => vec![theme],
                None => runtime.config.theme_bundles.keys().cloned().collect(),
            };
            let mut reports = Vec::new();
            for name in names {
                reports.push(
                    runtime
                        .check(&name)
                        .with_context(|| format!("theme bundle '{name}' failed validation"))?,
                );
            }
            if json {
                print_json(&reports)?;
            } else {
                for report in reports {
                    outputln!("ok: {}", report.theme);
                    print_plan(&report.capability_plan)?;
                    print_library_selections(&report.library_selections)?;
                }
            }
        }
        Command::Watch => unreachable!(),
    }
    Ok(())
}

fn print_plan(plan: &theme_core::CapabilityPlan) -> Result<()> {
    outputln!(
        "Capability plan: {}",
        if plan.complete { "complete" } else { "invalid" }
    );
    for claim in &plan.claims {
        outputln!(
            "  {}.{} <- {} ({:?}, {})",
            claim.application,
            claim.capability,
            claim.owner,
            claim.cadence,
            claim.boundary
        );
    }
    for gap in &plan.optional_gaps {
        outputln!("  optional gap: {}.{}", gap.application, gap.capability);
    }
    for claim in &plan.supported_but_unselected {
        outputln!(
            "  supported, not selected: {}.{} <- {} ({:?})",
            claim.application,
            claim.capability,
            claim.owner,
            claim.cadence
        );
    }
    for error in &plan.errors {
        outputln!("  error: {error}");
    }
    Ok(())
}

fn print_library_selections(selections: &[theme_core::LibrarySelectionStatus]) -> Result<()> {
    for selection in selections {
        outputln!(
            "  {}: {}/{} (source={}{}; cadence={:?}, effective={}{})",
            selection.slot,
            selection.library,
            selection.item,
            selection.source,
            selection
                .revision
                .as_ref()
                .map(|revision| format!("@{revision}"))
                .unwrap_or_default(),
            selection.selection_cadence,
            selection.effective,
            if selection.pending_session {
                ", pending session restart"
            } else {
                ""
            }
        );
    }
    Ok(())
}
