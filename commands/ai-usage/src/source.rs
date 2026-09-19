use crate::data::{Provider, ProviderId, Snapshot};
use std::{
    collections::BTreeMap,
    io::Read,
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    process::Stdio,
    sync::{Arc, Mutex, OnceLock},
    time::Duration,
};
use tokio::{task::JoinHandle, time::Instant};

/// One process-wide worker owns refresh admission, cached records, and retry pacing.
pub(crate) struct Source {
    state: Mutex<State>,
}

struct State {
    snapshot: Snapshot,
    due: Instant,
    worker: Option<JoinHandle<()>>,
}

impl State {
    fn new(now: Instant) -> Self {
        Self {
            snapshot: Snapshot::default(),
            due: now,
            worker: None,
        }
    }

    fn begin(&mut self, now: Instant, force: bool) -> bool {
        if self.snapshot.refreshing || (!force && now < self.due) {
            return false;
        }

        self.snapshot.refreshing = true;
        true
    }

    fn finish(&mut self, now: Instant, result: Result<(), String>) {
        self.snapshot.refreshing = false;

        if let Err(error) = result {
            if !self.snapshot.error.is_empty() {
                self.snapshot.error.push_str(" · ");
            }

            self.snapshot.error.push_str(&error);
        }

        self.due = now
            + Duration::from_secs(if self.snapshot.error.is_empty() {
                900
            } else {
                60
            });
    }

    fn records(&mut self, result: Result<(Vec<Provider>, RecordErrors), String>) {
        self.snapshot.loaded = true;

        match result {
            Ok((providers, errors)) => {
                // Malformed records retain the last good value and its original timestamp.
                let mut merged: BTreeMap<_, _> = self
                    .snapshot
                    .providers
                    .drain(..)
                    .map(|p| (p.id.clone(), p))
                    .collect();

                let present: Vec<_> = providers
                    .iter()
                    .map(|p| &p.id)
                    .chain(errors.iter().map(|(id, _)| id))
                    .collect();

                merged.retain(|id, _| present.contains(&id));

                for provider in providers {
                    merged.insert(provider.id.clone(), provider);
                }

                self.snapshot.providers = merged.into_values().collect();
                self.snapshot.error = errors
                    .into_iter()
                    .map(|(_, message)| message)
                    .collect::<Vec<_>>()
                    .join(" · ");
            }
            Err(error) => self.snapshot.error = error,
        }
    }
}

impl Source {
    pub(crate) fn shared() -> &'static Arc<Self> {
        static SOURCE: OnceLock<Arc<Source>> = OnceLock::new();
        SOURCE.get_or_init(|| {
            Arc::new(Self {
                state: Mutex::new(State::new(Instant::now())),
            })
        })
    }

    pub(crate) fn poll(self: &Arc<Self>, force: bool) -> Snapshot {
        let mut state = self.state.lock().expect("usage worker state poisoned");
        let now = Instant::now();

        if state.worker.as_ref().is_some_and(JoinHandle::is_finished) {
            state.worker.take();

            if state.snapshot.refreshing {
                state.finish(
                    now,
                    Err("Usage collection worker stopped unexpectedly".into()),
                );
            }
        }

        if state.begin(now, force) {
            let source = self.clone();
            state.worker = Some(tokio::spawn(async move {
                source.collect().await;
            }));
        }

        let mut snapshot = state.snapshot.clone();
        snapshot.now = chrono::Utc::now().timestamp() / 60 * 60;
        snapshot
    }

    async fn collect(&self) {
        // Cached records become visible while the network collection is still running.
        self.read().await;
        let result = Collector::run().await;
        self.read().await;
        self.state
            .lock()
            .expect("usage worker state poisoned")
            .finish(Instant::now(), result);
    }

    async fn read(&self) {
        let result = tokio::task::spawn_blocking(|| {
            Records::directory().and_then(|path| Records::read(&path))
        })
        .await
        .unwrap_or_else(|_| Err("Usage record worker failed".into()));
        self.state
            .lock()
            .expect("usage worker state poisoned")
            .records(result);
    }
}

pub(crate) struct Records;
type RecordErrors = Vec<(ProviderId, String)>;

impl Records {
    fn directory() -> Result<PathBuf, String> {
        let base = match std::env::var_os("XDG_STATE_HOME").filter(|p| !p.is_empty()) {
            Some(path) => PathBuf::from(path),
            None => PathBuf::from(std::env::var_os("HOME").ok_or("HOME is unavailable")?)
                .join(".local/state"),
        };
        Ok(base.join("omarchy/agents/usage"))
    }

    pub(crate) fn read(path: &Path) -> Result<(Vec<Provider>, RecordErrors), String> {
        let entries = match std::fs::read_dir(path) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok((Vec::new(), Vec::new()));
            }
            Err(_) => return Err("Cannot read Omarchy usage directory".into()),
        };

        let mut providers = Vec::new();
        let mut errors = Vec::new();
        let mut count = 0;

        for entry in entries {
            let entry = entry.map_err(|_| "Cannot list usage records")?;

            if entry.path().extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            count += 1;

            if count > 16 {
                return Err("More than 16 usage providers; refusing an unbounded scan".into());
            }

            let filename = entry.file_name();
            let stem = Path::new(&filename)
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or("Invalid usage filename")?;
            let id = ProviderId::parse(stem)?;

            match Self::record(&entry.path(), &id) {
                Ok(provider) => providers.push(provider),
                Err(error) => errors.push((id, format!("{stem}: {error}"))),
            }
        }

        providers.sort_by(|a, b| a.id.cmp(&b.id));
        errors.sort_by(|a, b| a.0.cmp(&b.0));
        Ok((providers, errors))
    }

    fn record(path: &Path, id: &ProviderId) -> Result<Provider, String> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(path)
            .map_err(|_| "Cannot open usage record as a regular file")?;

        if !file
            .metadata()
            .map_err(|_| "Cannot inspect usage record")?
            .is_file()
        {
            return Err("Usage record must be a regular file".into());
        }

        let mut bytes = Vec::new();
        file.take(512 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| "Cannot read usage record")?;

        if bytes.len() > 512 * 1024 {
            return Err("Usage record exceeds 512 KiB".into());
        }
        Provider::parse(&bytes, id)
    }
}

struct Collector;
impl Collector {
    async fn run() -> Result<(), String> {
        let mut command = tokio::process::Command::new("omarchy");
        command.args(["agent", "usage-update"]);
        Self::execute(command, Duration::from_secs(90)).await
    }

    async fn execute(
        mut command: tokio::process::Command,
        timeout: Duration,
    ) -> Result<(), String> {
        let mut child = command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .process_group(0)
            .kill_on_drop(true)
            .spawn()
            .map_err(|_| "Cannot start omarchy agent usage-update; install Omarchy".to_string())?;
        let mut group = ProcessGroup(Some(child.id().ok_or("Collector exited before startup")?));

        match tokio::time::timeout(timeout, child.wait()).await {
            Ok(Ok(status)) => {
                // The updater waits for its providers before exiting.
                group.0 = None;

                if status.success() {
                    Ok(())
                } else {
                    Err(format!(
                        "Collector failed ({status}); showing available records"
                    ))
                }
            }
            Ok(Err(_)) => Err("Cannot wait for usage collector".into()),
            Err(_) => {
                group.kill();
                let _ = child.wait().await;
                Err("Usage collection timed out; showing cached data".into())
            }
        }
    }
}

struct ProcessGroup(Option<u32>);
impl ProcessGroup {
    fn kill(&mut self) {
        if let Some(pid) = self.0.take() {
            // The updater forks collectors and app-server helpers. Cancel the whole group.
            unsafe {
                libc::kill(-(pid as i32), libc::SIGKILL);
            }
        }
    }
}

impl Drop for ProcessGroup {
    fn drop(&mut self) {
        self.kill();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::Fixture;

    #[tokio::test(start_paused = true)]
    async fn refreshes_coalesce_and_retry_without_losing_cached_records() {
        let mut state = State::new(Instant::now());

        assert!(state.begin(Instant::now(), false));
        assert!(!state.begin(Instant::now(), true));
        state.records(Ok((vec![Fixture::provider()], Vec::new())));
        state.finish(Instant::now(), Ok(()));
        tokio::time::advance(Duration::from_secs(899)).await;

        assert!(!state.begin(Instant::now(), false));
        tokio::time::advance(Duration::from_secs(1)).await;

        assert!(state.begin(Instant::now(), false));
        state.records(Ok((
            Vec::new(),
            vec![(
                ProviderId::parse("codex").unwrap(),
                "Malformed record".into(),
            )],
        )));
        state.finish(Instant::now(), Err("Collector unavailable".into()));

        assert_eq!(state.snapshot.providers[0], Fixture::provider());
        assert!(state.snapshot.error.contains("Malformed record"));
        assert!(state.snapshot.error.contains("Collector unavailable"));
        tokio::time::advance(Duration::from_secs(59)).await;

        assert!(!state.begin(Instant::now(), false));
        tokio::time::advance(Duration::from_secs(1)).await;

        assert!(state.begin(Instant::now(), false));
        state.records(Ok((vec![Fixture::provider()], Vec::new())));
        state.finish(Instant::now(), Ok(()));

        assert!(state.snapshot.error.is_empty());
        assert!(state.begin(Instant::now(), true));
        state.records(Err("Directory unavailable".into()));

        assert_eq!(state.snapshot.providers.len(), 1);
        state.records(Ok((Vec::new(), Vec::new())));

        assert!(state.snapshot.providers.is_empty());
    }

    #[tokio::test]
    async fn collector_reports_exit_failure_and_terminates_on_deadline() {
        let mut command = tokio::process::Command::new("/bin/sh");
        command.args(["-c", "exit 7"]);

        assert!(
            Collector::execute(command, Duration::from_secs(2))
                .await
                .unwrap_err()
                .contains("7")
        );

        let mut command = tokio::process::Command::new("/bin/sleep");
        command.arg("30");

        assert!(
            Collector::execute(command, Duration::from_millis(30))
                .await
                .unwrap_err()
                .contains("timed out")
        );
    }
}
