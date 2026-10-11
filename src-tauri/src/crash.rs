use crate::error::AppResult;
use crate::paths::DataPaths;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

const MAX_REPORT_BYTES: usize = 300 * 1024;
const LOG_TAIL_LINES: usize = 600;
const MAX_DETAILS: usize = 6;
const MAX_DETAIL_CHARS: usize = 220;
/// Files written a little before the game was started are still the previous run's.
const CLOCK_SLACK: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CrashKind {
    HeapTooLarge,
    OutOfMemory,
    WrongJava,
    MissingDependency,
    ModConflict,
    GraphicsDriver,
    ModProblem,
    Unknown,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CrashAnalysis {
    pub kind: CrashKind,
    /// The lines of the report that matter most, so the player does not have to read all of it.
    pub details: Vec<String>,
    pub report_path: Option<String>,
}

fn collect<'a>(text: &'a str, wanted: impl Fn(&str) -> bool) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for line in text.lines().map(str::trim).filter(|line| !line.is_empty() && wanted(line)) {
        let shortened: String = line.chars().take(MAX_DETAIL_CHARS).collect();
        if !found.contains(&shortened) {
            found.push(shortened);
        }
        if found.len() == MAX_DETAILS {
            break;
        }
    }
    found
}

fn any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

/// Works out what went wrong from the text of a crash report or a game log.
pub fn analyze_text(text: &str) -> (CrashKind, Vec<String>) {
    if any(text, &["Could not reserve enough space", "Invalid maximum heap size", "Error occurred during initialization of VM"]) {
        return (CrashKind::HeapTooLarge, collect(text, |line| line.contains("heap") || line.contains("reserve") || line.contains("initialization of VM")));
    }
    if any(text, &["java.lang.OutOfMemoryError", "OutOfMemoryError:"]) {
        return (CrashKind::OutOfMemory, collect(text, |line| line.contains("OutOfMemoryError")));
    }
    if any(text, &["UnsupportedClassVersionError", "compiled by a more recent version of the Java Runtime"]) {
        return (CrashKind::WrongJava, collect(text, |line| line.contains("UnsupportedClassVersionError") || line.contains("class file version")));
    }
    let missing = |line: &str| {
        line.contains("which is missing") || line.contains("Missing or unsupported mandatory dependencies") || line.contains("Requested by") || (line.contains("requires") && line.contains("mod"))
    };
    if text.lines().any(|line| missing(line)) {
        return (CrashKind::MissingDependency, collect(text, missing));
    }
    let conflict = |line: &str| line.contains("conflicts with") || line.contains("is incompatible with") || line.contains("Incompatible mods found");
    if text.lines().any(|line| conflict(line)) {
        return (CrashKind::ModConflict, collect(text, conflict));
    }
    if any(text, &["GLFW error", "Pixel format not accelerated", "does not appear to support OpenGL", "WGL:"]) {
        return (CrashKind::GraphicsDriver, collect(text, |line| line.contains("GLFW") || line.contains("OpenGL") || line.contains("Pixel format") || line.contains("WGL")));
    }
    let problem = |line: &str| {
        line.contains("Mixin apply") || line.contains("MixinApply") || line.contains("InvalidMixin") || line.contains("MixinTransformer") || line.contains("NoSuchMethodError") || line.contains("NoClassDefFoundError") || line.contains("ClassNotFoundException") || line.starts_with("Caused by:")
    };
    if text.lines().any(|line| problem(line.trim())) {
        return (CrashKind::ModProblem, collect(text, |line| problem(line)));
    }
    (CrashKind::Unknown, collect(text, |line| line.starts_with("Description:") || line.starts_with("java.lang.") || line.starts_with("Exception in thread")))
}

fn newest_since(folder: &Path, prefix: &str, suffix: &str, since: SystemTime) -> Option<PathBuf> {
    fs::read_dir(folder)
        .ok()?
        .flatten()
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            name.starts_with(prefix) && name.ends_with(suffix)
        })
        .filter_map(|entry| Some((entry.metadata().ok()?.modified().ok()?, entry.path())))
        .filter(|(modified, _)| *modified >= since)
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
}

fn read_limited(path: &Path) -> String {
    let bytes = fs::read(path).unwrap_or_default();
    String::from_utf8_lossy(&bytes[..bytes.len().min(MAX_REPORT_BYTES)]).into_owned()
}

fn log_tail(path: &Path) -> String {
    let text = read_limited(path);
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(LOG_TAIL_LINES)..].join("\n")
}

/// Looks at what the game left behind after a run that started at `started`: a crash report, a Java crash file or the log.
pub fn analyze(paths: &DataPaths, instance_id: &str, started: SystemTime) -> AppResult<Option<CrashAnalysis>> {
    let game_dir = paths.instance_game_dir(instance_id);
    let since = started.checked_sub(CLOCK_SLACK).unwrap_or(started);
    let report = newest_since(&game_dir.join("crash-reports"), "crash-", ".txt", since)
        .map(|path| (read_limited(&path), Some(path)))
        .or_else(|| newest_since(&game_dir, "hs_err_pid", ".log", since).map(|path| (read_limited(&path), Some(path))))
        .or_else(|| {
            let log = game_dir.join("logs").join("latest.log");
            log.is_file().then(|| (log_tail(&log), Some(log)))
        });
    let Some((text, path)) = report else { return Ok(None) };
    let (kind, details) = analyze_text(&text);
    Ok(Some(CrashAnalysis { kind, details, report_path: path.map(|path| path.to_string_lossy().into_owned()) }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_a_heap_that_is_bigger_than_the_computer_can_give() {
        let text = "Error occurred during initialization of VM\nCould not reserve enough space for 20971520KB object heap";
        let (kind, details) = analyze_text(text);
        assert_eq!(kind, CrashKind::HeapTooLarge);
        assert!(details.iter().any(|line| line.contains("reserve")));
    }

    #[test]
    fn recognises_running_out_of_memory() {
        let text = "Description: Exception in server tick loop\n\njava.lang.OutOfMemoryError: Java heap space\n\tat net.minecraft.A.b(SourceFile:1)";
        assert_eq!(analyze_text(text).0, CrashKind::OutOfMemory);
    }

    #[test]
    fn recognises_the_wrong_java_version() {
        let text = "java.lang.UnsupportedClassVersionError: net/fabricmc/loader/Main has been compiled by a more recent version of the Java Runtime (class file version 65.0)";
        assert_eq!(analyze_text(text).0, CrashKind::WrongJava);
    }

    #[test]
    fn reads_missing_dependencies_from_fabric_and_forge_reports() {
        let fabric = "Incompatible mods found!\n - Mod 'Sodium Extra' (sodium-extra) 0.5.4 requires any version of mod sodium, which is missing!";
        let (kind, details) = analyze_text(fabric);
        assert_eq!(kind, CrashKind::MissingDependency);
        assert!(details.iter().any(|line| line.contains("sodium")));

        let forge = "Missing or unsupported mandatory dependencies:\n\tMod ID: 'geckolib', Requested by: 'alexsmobs', Expected range: '[4.0,)'";
        assert_eq!(analyze_text(forge).0, CrashKind::MissingDependency);
    }

    #[test]
    fn recognises_conflicting_mods_graphics_trouble_and_mod_bugs() {
        assert_eq!(analyze_text("Mod 'A' (a) conflicts with mod 'B' (b), and cannot be loaded together").0, CrashKind::ModConflict);
        assert_eq!(analyze_text("GLFW error 65542: WGL: The driver does not appear to support OpenGL").0, CrashKind::GraphicsDriver);
        let (kind, details) = analyze_text("Mixin apply failed examplemod.mixins.json:ThingMixin\nCaused by: java.lang.NoSuchMethodError: net.minecraft.Foo.bar()V");
        assert_eq!(kind, CrashKind::ModProblem);
        assert!(details.len() >= 2);
    }

    #[test]
    fn falls_back_to_the_description_when_nothing_is_recognised() {
        let (kind, details) = analyze_text("---- Minecraft Crash Report ----\nDescription: Ticking entity\n\nsomething odd");
        assert_eq!(kind, CrashKind::Unknown);
        assert_eq!(details, vec!["Description: Ticking entity".to_owned()]);
    }

    #[test]
    fn picks_the_report_of_this_run_and_ignores_older_ones() {
        let paths = DataPaths::new(std::env::temp_dir().join(format!("kairos-crash-{}", std::process::id())));
        let _ = fs::remove_dir_all(paths.root());
        let reports = paths.instance_game_dir("one").join("crash-reports");
        fs::create_dir_all(&reports).unwrap();

        let started = SystemTime::now();
        assert!(analyze(&paths, "one", started + Duration::from_secs(3600)).unwrap().is_none());

        fs::write(reports.join("crash-2026-10-11_12.00.00-client.txt"), "java.lang.OutOfMemoryError: Java heap space").unwrap();
        let found = analyze(&paths, "one", started).unwrap().unwrap();
        assert_eq!(found.kind, CrashKind::OutOfMemory);
        assert!(found.report_path.unwrap().ends_with("client.txt"));
        assert!(analyze(&paths, "one", started + Duration::from_secs(3600)).unwrap().is_none());

        let log = paths.instance_game_dir("two").join("logs");
        fs::create_dir_all(&log).unwrap();
        fs::write(log.join("latest.log"), "GLFW error 65542: oops").unwrap();
        assert_eq!(analyze(&paths, "two", started).unwrap().unwrap().kind, CrashKind::GraphicsDriver);
        let _ = fs::remove_dir_all(paths.root());
    }
}
