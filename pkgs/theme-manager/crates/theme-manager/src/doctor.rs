use crate::{health, registry::Registry, runtime::Runtime};
use anyhow::{Result, bail};
use serde::Serialize;
use std::{
    collections::BTreeSet,
    fs,
    io::{self, Write},
    path::Path,
};
use theme_core::{Config, GENERATED_MARKER, paths::canonical_output_path, state};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
enum CheckStatus {
    Ok,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize)]
struct Check {
    code: String,
    status: CheckStatus,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target: Option<String>,
}

#[derive(Debug, Serialize)]
struct DoctorReport {
    schema: u32,
    ok: bool,
    checks: Vec<Check>,
}

impl DoctorReport {
    fn new() -> Self {
        Self {
            schema: 1,
            ok: true,
            checks: Vec::new(),
        }
    }

    fn push(&mut self, status: CheckStatus, code: &str, message: impl Into<String>) {
        if matches!(status, CheckStatus::Error) {
            self.ok = false;
        }
        self.checks.push(Check {
            code: code.into(),
            status,
            message: message.into(),
            path: None,
            target: None,
        });
    }

    fn contextual(
        &mut self,
        status: CheckStatus,
        code: &str,
        message: impl Into<String>,
        path: Option<&Path>,
        target: Option<&str>,
    ) {
        if matches!(status, CheckStatus::Error) {
            self.ok = false;
        }
        self.checks.push(Check {
            code: code.into(),
            status,
            message: message.into(),
            path: path.map(|path| path.display().to_string()),
            target: target.map(str::to_string),
        });
    }
}

fn marker_owned(contents: &[u8]) -> bool {
    let prefix = String::from_utf8_lossy(&contents[..contents.len().min(512)]);
    let header = prefix.lines().next();
    header == Some(GENERATED_MARKER)
        || header.and_then(|line| line.strip_prefix("# ")) == Some(GENERATED_MARKER)
        || header.and_then(|line| line.strip_prefix("// ")) == Some(GENERATED_MARKER)
}

fn check_output_parent(report: &mut DoctorReport, path: &Path, target: &str) {
    let mut ancestor = path.parent();
    while let Some(candidate) = ancestor {
        match fs::metadata(candidate) {
            Ok(metadata) => {
                if !metadata.is_dir() {
                    report.contextual(
                        CheckStatus::Error,
                        "output.parent_invalid",
                        "nearest existing output parent is not a directory",
                        Some(candidate),
                        Some(target),
                    );
                }
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if metadata.permissions().mode() & 0o222 == 0 {
                        report.contextual(
                            CheckStatus::Error,
                            "output.parent_readonly",
                            "nearest existing output parent has no write bits",
                            Some(candidate),
                            Some(target),
                        );
                    }
                }
                return;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                ancestor = candidate.parent();
            }
            Err(error) => {
                report.contextual(
                    CheckStatus::Error,
                    "output.parent_inspect",
                    error.to_string(),
                    Some(candidate),
                    Some(target),
                );
                return;
            }
        }
    }
}

fn build_report(requested_config: Option<&Path>) -> DoctorReport {
    let mut report = DoctorReport::new();
    let (config, config_path) = match Config::load(requested_config) {
        Ok(value) => value,
        Err(error) => {
            report.push(CheckStatus::Error, "config.load", format!("{error:#}"));
            return report;
        }
    };
    report.contextual(
        CheckStatus::Ok,
        "config.load",
        "configuration loaded",
        Some(&config_path),
        None,
    );

    match Registry::load(&config) {
        Ok(registry) => {
            if registry.diagnostics().is_empty() {
                report.push(
                    CheckStatus::Ok,
                    "plugins.discovery",
                    "plugin discovery succeeded",
                );
            } else {
                for diagnostic in registry.diagnostics() {
                    report.push(CheckStatus::Error, "plugins.discovery", diagnostic);
                }
            }
            for extension in registry.extensions() {
                match theme_extension_luau::validate_extension(extension) {
                    Ok(()) => report.push(
                        CheckStatus::Ok,
                        "plugin.valid",
                        format!("plugin '{}' is valid", extension.adapter_id),
                    ),
                    Err(error) => report.contextual(
                        CheckStatus::Error,
                        "plugin.invalid",
                        format!("{error:#}"),
                        Some(&extension.manifest_path),
                        None,
                    ),
                }
            }
        }
        Err(error) => report.push(
            CheckStatus::Error,
            "plugins.load",
            format!("failed to load plugins: {error:#}"),
        ),
    }

    let runtime = match Runtime::load(Some(&config_path)) {
        Ok(runtime) => runtime,
        Err(error) => {
            report.push(CheckStatus::Error, "runtime.load", format!("{error:#}"));
            return report;
        }
    };
    report.push(CheckStatus::Ok, "runtime.load", "runtime initialized");

    let profiles = match runtime.engine.list_profiles() {
        Ok(profiles) => profiles,
        Err(error) => {
            report.push(CheckStatus::Error, "profiles.list", format!("{error:#}"));
            return report;
        }
    };
    if profiles.is_empty() {
        report.push(
            CheckStatus::Error,
            "profiles.empty",
            "no profiles were found",
        );
    }
    for profile in &profiles {
        match runtime.engine.validate_profile(&profile.name) {
            Ok(()) => report.push(
                CheckStatus::Ok,
                "profile.valid",
                format!("profile '{}' is valid", profile.name),
            ),
            Err(error) => report.push(
                CheckStatus::Error,
                "profile.invalid",
                format!("profile '{}': {error:#}", profile.name),
            ),
        }
    }

    match runtime.engine.active_profile() {
        Ok(Some(active)) if profiles.iter().any(|profile| profile.name == active) => report.push(
            CheckStatus::Ok,
            "profile.active",
            format!("active profile is '{active}'"),
        ),
        Ok(Some(active)) => report.push(
            CheckStatus::Error,
            "profile.active_missing",
            format!("active/default profile '{active}' does not exist"),
        ),
        Ok(None) => report.push(
            CheckStatus::Error,
            "profile.active_missing",
            "no active or default profile is configured",
        ),
        Err(error) => report.push(CheckStatus::Error, "profile.active", format!("{error:#}")),
    }

    let state_file = match runtime.engine.state_file() {
        Ok(path) => path,
        Err(error) => {
            report.push(CheckStatus::Error, "state.path", format!("{error:#}"));
            return report;
        }
    };
    let current_state = match state::read(&state_file) {
        Ok(state) => state,
        Err(error) => {
            report.contextual(
                CheckStatus::Error,
                "state.read",
                format!("{error:#}"),
                Some(&state_file),
                None,
            );
            return report;
        }
    };

    let mut configured_outputs = BTreeSet::new();
    for (target, path) in runtime.diagnostic_outputs() {
        check_output_parent(&mut report, &path, &target);
        let contents = match fs::read(&path) {
            Ok(contents) => contents,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                report.contextual(
                    CheckStatus::Ok,
                    "output.absent",
                    "output does not exist yet",
                    Some(&path),
                    Some(&target),
                );
                continue;
            }
            Err(error) => {
                report.contextual(
                    CheckStatus::Error,
                    "output.read",
                    error.to_string(),
                    Some(&path),
                    Some(&target),
                );
                continue;
            }
        };
        let key = match canonical_output_path(&path) {
            Ok(path) => path.to_string_lossy().into_owned(),
            Err(error) => {
                report.contextual(
                    CheckStatus::Error,
                    "output.path",
                    format!("{error:#}"),
                    Some(&path),
                    Some(&target),
                );
                continue;
            }
        };
        configured_outputs.insert(key.clone());
        let hash = state::content_hash(&contents);
        match current_state.generated_outputs.get(&key) {
            Some(owner) if owner.target == target && owner.hash == hash => report.contextual(
                CheckStatus::Ok,
                "output.owned",
                "output matches its ownership record",
                Some(&path),
                Some(&target),
            ),
            None if marker_owned(&contents) => report.contextual(
                CheckStatus::Warning,
                "output.legacy_marker",
                "output has a legacy marker and will be adopted on the next apply",
                Some(&path),
                Some(&target),
            ),
            _ => report.contextual(
                CheckStatus::Error,
                "output.conflict",
                "output is modified or not owned by theme-manager",
                Some(&path),
                Some(&target),
            ),
        }
    }

    for (path, owner) in &current_state.generated_outputs {
        if !configured_outputs.contains(path) {
            report.contextual(
                CheckStatus::Warning,
                "output.stale_ownership",
                format!(
                    "ownership record belongs to unconfigured target '{}'",
                    owner.target
                ),
                Some(Path::new(path)),
                Some(&owner.target),
            );
        }
    }

    let health_path = health::path_for_state(&state_file);
    match health::read(&health_path) {
        Ok(None) => report.contextual(
            CheckStatus::Warning,
            "watcher.status_missing",
            "watcher health is unavailable; the service may be disabled",
            Some(&health_path),
            None,
        ),
        Err(error) => report.contextual(
            CheckStatus::Error,
            "watcher.status_invalid",
            format!("{error:#}"),
            Some(&health_path),
            None,
        ),
        Ok(Some(status)) if status.schema != 1 => report.contextual(
            CheckStatus::Error,
            "watcher.status_schema",
            format!("unsupported watcher health schema {}", status.schema),
            Some(&health_path),
            None,
        ),
        Ok(Some(status)) => {
            if !health::process_is_alive(status.pid) {
                report.contextual(
                    CheckStatus::Warning,
                    "watcher.not_running",
                    format!("watcher process {} is not running", status.pid),
                    Some(&health_path),
                    None,
                );
            } else if health::now_ms().saturating_sub(status.last_attempt_ms) > 90_000 {
                report.contextual(
                    CheckStatus::Error,
                    "watcher.stale",
                    "watcher heartbeat is older than 90 seconds",
                    Some(&health_path),
                    None,
                );
            } else if let Some(error) = status.last_error {
                report.contextual(
                    CheckStatus::Error,
                    "watcher.apply_failed",
                    error,
                    Some(&health_path),
                    None,
                );
            } else {
                report.contextual(
                    CheckStatus::Ok,
                    "watcher.healthy",
                    "watcher is running and healthy",
                    Some(&health_path),
                    None,
                );
            }
        }
    }

    report
}

pub fn run(config: Option<&Path>, json: bool) -> Result<()> {
    let report = build_report(config);
    let rendered = if json {
        format!("{}\n", serde_json::to_string_pretty(&report)?)
    } else {
        let mut rendered = String::new();
        for check in &report.checks {
            let marker = match check.status {
                CheckStatus::Ok => "ok",
                CheckStatus::Warning => "warn",
                CheckStatus::Error => "error",
            };
            rendered.push_str(&format!(
                "{marker:<5} {:<28} {}\n",
                check.code, check.message
            ));
        }
        rendered
    };
    if let Err(error) = write!(io::stdout().lock(), "{rendered}") {
        if error.kind() == io::ErrorKind::BrokenPipe {
            return Ok(());
        }
        return Err(error.into());
    }
    if !report.ok {
        bail!("doctor found one or more errors");
    }
    Ok(())
}

pub fn validation_summary(config: Option<&Path>) -> Result<String> {
    let report = build_report(config);
    let errors = report
        .checks
        .iter()
        .filter(|check| matches!(check.status, CheckStatus::Error))
        .map(|check| format!("{}: {}", check.code, check.message))
        .collect::<Vec<_>>();
    if errors.is_empty() {
        let warnings = report
            .checks
            .iter()
            .filter(|check| matches!(check.status, CheckStatus::Warning))
            .count();
        Ok(format!(
            "Configuration is healthy ({} check(s), {warnings} warning(s)).",
            report.checks.len()
        ))
    } else {
        bail!("{}", errors.join("; "))
    }
}

#[cfg(test)]
mod tests {
    use super::{CheckStatus, DoctorReport};

    #[test]
    fn warnings_do_not_fail_the_report() {
        let mut report = DoctorReport::new();
        report.push(CheckStatus::Warning, "warning", "warning");
        assert!(report.ok);
        report.push(CheckStatus::Error, "error", "error");
        assert!(!report.ok);
    }
}
