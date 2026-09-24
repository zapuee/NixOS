use crate::{
    fs::{create_temp_file, sync_parent, write_destination},
    state::{self, State},
};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_TRANSACTION: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub enum Operation {
    Write {
        path: PathBuf,
        contents: String,
        expected_hash: Option<String>,
    },
    Remove {
        path: PathBuf,
        expected_hash: String,
    },
}

#[derive(Debug, Default)]
pub struct CommitReport {
    pub written: Vec<PathBuf>,
    pub removed: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Journal {
    schema: u32,
    id: String,
    operations: Vec<JournalOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum JournalOperation {
    Write {
        logical_path: PathBuf,
        destination: PathBuf,
        staged: PathBuf,
        backup: PathBuf,
        previous_hash: Option<String>,
        new_hash: String,
    },
    Remove {
        logical_path: PathBuf,
        destination: PathBuf,
        backup: PathBuf,
        previous_hash: String,
    },
}

impl JournalOperation {
    fn destination(&self) -> &Path {
        match self {
            Self::Write { destination, .. } | Self::Remove { destination, .. } => destination,
        }
    }

    fn backup(&self) -> &Path {
        match self {
            Self::Write { backup, .. } | Self::Remove { backup, .. } => backup,
        }
    }
}

fn transaction_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let sequence = NEXT_TRANSACTION.fetch_add(1, Ordering::Relaxed);
    format!("{}-{nanos}-{sequence}", std::process::id())
}

fn journal_path(state_file: &Path) -> PathBuf {
    let name = state_file
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("state.toml");
    state_file.with_file_name(format!(".{name}.transaction.toml"))
}

fn sibling_path(destination: &Path, id: &str, suffix: &str) -> PathBuf {
    let name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("output");
    destination.with_file_name(format!(".{name}.{id}.{suffix}"))
}

fn regular_file_hash(path: &Path) -> Result<Option<String>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            let metadata = fs::metadata(path)
                .with_context(|| format!("failed to inspect output symlink {}", path.display()))?;
            if !metadata.is_file() {
                bail!(
                    "output {} does not resolve to a regular file",
                    path.display()
                );
            }
        }
        Ok(metadata) if !metadata.is_file() => {
            bail!("output {} is not a regular file", path.display());
        }
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to inspect output {}", path.display()));
        }
    }
    let contents =
        fs::read(path).with_context(|| format!("failed to read output {}", path.display()))?;
    Ok(Some(state::content_hash(contents)))
}

fn prepare(id: &str, operations: Vec<Operation>) -> Result<Vec<JournalOperation>> {
    let mut prepared = Vec::with_capacity(operations.len());
    for operation in operations {
        match prepare_one(id, operation) {
            Ok(operation) => prepared.push(operation),
            Err(error) => {
                for operation in &prepared {
                    if let JournalOperation::Write { staged, .. } = operation {
                        let _ = fs::remove_file(staged);
                    }
                }
                return Err(error);
            }
        }
    }
    Ok(prepared)
}

fn prepare_one(id: &str, operation: Operation) -> Result<JournalOperation> {
    Ok(match operation {
        Operation::Write {
            path,
            contents,
            expected_hash,
        } => {
            let destination = write_destination(&path)?;
            if let Some(parent) = destination
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
            {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
            let previous_hash = regular_file_hash(&destination)?;
            if previous_hash != expected_hash {
                bail!(
                    "output {} changed while the transaction was being prepared",
                    path.display()
                );
            }
            let name = destination
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("output");
            let (staged, mut file) = create_temp_file(&destination, name)?;
            if let Err(error) = file
                .write_all(contents.as_bytes())
                .and_then(|()| file.sync_all())
            {
                drop(file);
                let _ = fs::remove_file(&staged);
                return Err(error)
                    .with_context(|| format!("failed to stage output {}", destination.display()));
            }
            drop(file);
            if let Ok(metadata) = fs::metadata(&destination)
                && let Err(error) = fs::set_permissions(&staged, metadata.permissions())
            {
                let _ = fs::remove_file(&staged);
                return Err(error).with_context(|| {
                    format!(
                        "failed to preserve permissions for {}",
                        destination.display()
                    )
                });
            }
            JournalOperation::Write {
                logical_path: path,
                backup: sibling_path(&destination, id, "backup"),
                destination,
                staged,
                previous_hash,
                new_hash: state::content_hash(contents),
            }
        }
        Operation::Remove {
            path,
            expected_hash,
        } => {
            let previous_hash = regular_file_hash(&path)?.with_context(|| {
                format!(
                    "output {} disappeared before it could be removed",
                    path.display()
                )
            })?;
            if previous_hash != expected_hash {
                bail!(
                    "output {} changed while the transaction was being prepared",
                    path.display()
                );
            }
            JournalOperation::Remove {
                logical_path: path.clone(),
                backup: sibling_path(&path, id, "backup"),
                destination: path,
                previous_hash,
            }
        }
    })
}

fn write_journal(path: &Path, journal: &Journal) -> Result<()> {
    let raw = toml::to_string_pretty(journal).context("failed to serialize transaction journal")?;
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create journal directory {}", parent.display()))?;
    }
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("journal");
    let (temporary, mut file) = create_temp_file(path, name)?;
    if let Err(error) = file
        .write_all(raw.as_bytes())
        .and_then(|()| file.sync_all())
    {
        drop(file);
        let _ = fs::remove_file(&temporary);
        return Err(error).context("failed to write transaction journal");
    }
    drop(file);
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(error)
            .with_context(|| format!("failed to install transaction journal {}", path.display()));
    }
    sync_parent(path)?;
    Ok(())
}

fn remove_if_exists(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).with_context(|| format!("failed to remove {}", path.display())),
    }
}

fn current_matches(path: &Path, expected: Option<&str>) -> Result<bool> {
    Ok(regular_file_hash(path)?.as_deref() == expected)
}

fn commit_files(journal: &Journal) -> Result<CommitReport> {
    let mut report = CommitReport::default();
    for operation in &journal.operations {
        match operation {
            JournalOperation::Write {
                logical_path,
                destination,
                staged,
                backup,
                previous_hash,
                ..
            } => {
                if !current_matches(destination, previous_hash.as_deref())? {
                    bail!(
                        "output {} changed while the transaction was being prepared",
                        logical_path.display()
                    );
                }
                if previous_hash.is_some() {
                    fs::rename(destination, backup).with_context(|| {
                        format!("failed to back up output {}", logical_path.display())
                    })?;
                }
                if let Err(error) = fs::rename(staged, destination) {
                    if previous_hash.is_some() {
                        let _ = fs::rename(backup, destination);
                    }
                    return Err(error).with_context(|| {
                        format!("failed to replace output {}", logical_path.display())
                    });
                }
                sync_parent(destination)?;
                report.written.push(logical_path.clone());
            }
            JournalOperation::Remove {
                logical_path,
                destination,
                backup,
                previous_hash,
            } => {
                if !current_matches(destination, Some(previous_hash))? {
                    bail!(
                        "output {} changed while the transaction was being prepared",
                        logical_path.display()
                    );
                }
                fs::rename(destination, backup).with_context(|| {
                    format!("failed to stage removal of {}", logical_path.display())
                })?;
                sync_parent(destination)?;
                report.removed.push(logical_path.clone());
            }
        }
    }
    Ok(report)
}

fn rollback(journal: &Journal) -> Result<()> {
    let mut failures = Vec::new();
    for operation in journal.operations.iter().rev() {
        let destination = operation.destination();
        let backup = operation.backup();
        if backup.exists() {
            if destination.exists()
                && let Err(error) = fs::remove_file(destination)
            {
                failures.push(format!("{}: {error}", destination.display()));
                continue;
            }
            if let Err(error) = fs::rename(backup, destination) {
                failures.push(format!("{}: {error}", destination.display()));
            } else if let Err(error) = sync_parent(destination) {
                failures.push(format!("{}: {error:#}", destination.display()));
            }
        } else if let JournalOperation::Write {
            previous_hash: None,
            new_hash,
            staged,
            ..
        } = operation
            && !staged.exists()
            && current_matches(destination, Some(new_hash)).unwrap_or(false)
            && let Err(error) = fs::remove_file(destination)
        {
            failures.push(format!("{}: {error}", destination.display()));
        } else if !destination.exists()
            && let Err(error) = sync_parent(destination)
        {
            failures.push(format!("{}: {error:#}", destination.display()));
        }
        if let JournalOperation::Write { staged, .. } = operation {
            let _ = remove_if_exists(staged);
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        bail!(
            "transaction rollback was incomplete: {}",
            failures.join("; ")
        )
    }
}

fn finalize(journal_path: &Path, journal: &Journal) -> Result<()> {
    for operation in &journal.operations {
        remove_if_exists(operation.backup())?;
        if let JournalOperation::Write { staged, .. } = operation {
            remove_if_exists(staged)?;
        }
    }
    remove_if_exists(journal_path)
}

/// Recover an interrupted transaction. Call while holding the state lock.
pub fn recover(state_file: &Path) -> Result<()> {
    let path = journal_path(state_file);
    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error).context("failed to read transaction journal"),
    };
    let journal: Journal = toml::from_str(&raw).context("failed to parse transaction journal")?;
    if journal.schema != 1 {
        bail!("unsupported transaction journal schema {}", journal.schema);
    }
    let committed = state::read(state_file)?.last_transaction.as_deref() == Some(&journal.id);
    if committed {
        finalize(&path, &journal)
    } else {
        rollback(&journal)?;
        remove_if_exists(&path)
    }
}

/// Commit output mutations and their resulting state as one recoverable transaction.
/// The caller must hold the state lock.
pub fn commit(
    state_file: &Path,
    operations: Vec<Operation>,
    state: &mut State,
) -> Result<CommitReport> {
    recover(state_file)?;
    if operations.is_empty() {
        state::write(state_file, state)?;
        return Ok(CommitReport::default());
    }

    let id = transaction_id();
    let prepared = match prepare(&id, operations) {
        Ok(prepared) => prepared,
        Err(error) => return Err(error).context("failed to prepare output transaction"),
    };
    let journal = Journal {
        schema: 1,
        id: id.clone(),
        operations: prepared,
    };
    let path = journal_path(state_file);
    if let Err(error) = write_journal(&path, &journal) {
        let _ = rollback(&journal);
        return Err(error);
    }

    let report = match commit_files(&journal) {
        Ok(report) => report,
        Err(error) => {
            rollback(&journal).with_context(|| format!("output transaction failed ({error:#})"))?;
            remove_if_exists(&path)?;
            return Err(error);
        }
    };

    state.last_transaction = Some(id);
    if let Err(error) = state::write(state_file, state) {
        rollback(&journal).with_context(|| format!("failed to commit state ({error:#})"))?;
        remove_if_exists(&path)?;
        return Err(error);
    }
    finalize(&path, &journal).context("outputs committed, but transaction cleanup failed")?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::{
        Journal, Operation, commit, commit_files, journal_path, prepare, recover, write_journal,
    };
    use crate::state::{self, State};
    use std::fs;

    #[test]
    fn writes_and_removals_commit_with_state() {
        let root = std::env::temp_dir().join(format!(
            "theme-manager-transaction-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let replaced = root.join("replace.conf");
        let removed = root.join("remove.conf");
        let state_file = root.join("state.toml");
        fs::write(&replaced, "old").unwrap();
        fs::write(&removed, "remove").unwrap();
        let mut state = State {
            active_profile: Some("glass".into()),
            ..State::default()
        };
        let report = commit(
            &state_file,
            vec![
                Operation::Write {
                    path: replaced.clone(),
                    contents: "new".into(),
                    expected_hash: Some(state::content_hash("old")),
                },
                Operation::Remove {
                    path: removed.clone(),
                    expected_hash: state::content_hash("remove"),
                },
            ],
            &mut state,
        )
        .unwrap();
        assert_eq!(report.written, vec![replaced.clone()]);
        assert_eq!(report.removed, vec![removed.clone()]);
        assert_eq!(fs::read_to_string(replaced).unwrap(), "new");
        assert!(!removed.exists());
        assert_eq!(
            state::read(&state_file).unwrap().active_profile.as_deref(),
            Some("glass")
        );
        recover(&state_file).unwrap();
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn recovery_rolls_back_files_when_state_did_not_commit() {
        let root = std::env::temp_dir().join(format!(
            "theme-manager-transaction-rollback-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let output = root.join("output.conf");
        let state_file = root.join("state.toml");
        fs::write(&output, "old").unwrap();
        let id = "interrupted".to_string();
        let journal = Journal {
            schema: 1,
            id: id.clone(),
            operations: prepare(
                &id,
                vec![Operation::Write {
                    path: output.clone(),
                    contents: "new".into(),
                    expected_hash: Some(state::content_hash("old")),
                }],
            )
            .unwrap(),
        };
        write_journal(&journal_path(&state_file), &journal).unwrap();
        commit_files(&journal).unwrap();
        assert_eq!(fs::read_to_string(&output).unwrap(), "new");

        recover(&state_file).unwrap();
        assert_eq!(fs::read_to_string(&output).unwrap(), "old");
        assert!(!journal_path(&state_file).exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn recovery_finalizes_files_when_state_committed() {
        let root = std::env::temp_dir().join(format!(
            "theme-manager-transaction-finalize-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let output = root.join("output.conf");
        let state_file = root.join("state.toml");
        fs::write(&output, "old").unwrap();
        let id = "committed".to_string();
        let journal = Journal {
            schema: 1,
            id: id.clone(),
            operations: prepare(
                &id,
                vec![Operation::Write {
                    path: output.clone(),
                    contents: "new".into(),
                    expected_hash: Some(state::content_hash("old")),
                }],
            )
            .unwrap(),
        };
        write_journal(&journal_path(&state_file), &journal).unwrap();
        commit_files(&journal).unwrap();
        state::write(
            &state_file,
            &State {
                last_transaction: Some(id),
                ..State::default()
            },
        )
        .unwrap();

        recover(&state_file).unwrap();
        assert_eq!(fs::read_to_string(&output).unwrap(), "new");
        assert!(!journal_path(&state_file).exists());
        fs::remove_dir_all(root).unwrap();
    }
}
