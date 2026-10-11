use crate::instances::model::LoaderKind;
use serde::Serialize;

/// Well known Java settings for the game: short pauses of the garbage collector and a steady heap.
const JAVA_FLAGS: [&str; 10] = [
    "-XX:+UseG1GC",
    "-XX:+ParallelRefProcEnabled",
    "-XX:MaxGCPauseMillis=130",
    "-XX:+UnlockExperimentalVMOptions",
    "-XX:+DisableExplicitGC",
    "-XX:G1NewSizePercent=30",
    "-XX:G1MaxNewSizePercent=40",
    "-XX:G1HeapRegionSize=8M",
    "-XX:G1ReservePercent=20",
    "-XX:InitiatingHeapOccupancyPercent=15",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoostItem {
    /// The catalogue name of the mod.
    pub project: String,
    pub title: Option<String>,
    pub installed: bool,
    /// Why a mod was skipped, e.g. no version of it exists for this game version.
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoostReport {
    pub items: Vec<BoostItem>,
    pub memory_mb: u32,
    pub java_flags_added: bool,
}

/// The mods that make the game run noticeably better on the given loader.
pub fn performance_mods(loader: LoaderKind) -> &'static [&'static str] {
    match loader {
        LoaderKind::Vanilla => &[],
        LoaderKind::Fabric | LoaderKind::NeoForge => &["sodium", "lithium", "ferrite-core", "modernfix", "entityculling"],
        LoaderKind::Quilt => &["sodium", "lithium", "ferrite-core", "entityculling"],
        LoaderKind::Forge => &["embeddium", "ferrite-core", "modernfix", "entityculling"],
    }
}

fn picks_another_collector(arguments: &[String]) -> bool {
    arguments.iter().any(|argument| argument.starts_with("-XX:+Use") && argument.ends_with("GC") && argument != "-XX:+UseG1GC")
}

fn key(argument: &str) -> &str {
    argument.split('=').next().unwrap_or(argument)
}

/// Adds the Java settings that are missing, never touching what the player already set. Someone who chose a
/// different garbage collector keeps their setup untouched.
pub fn with_java_flags(existing: &[String]) -> Vec<String> {
    if picks_another_collector(existing) {
        return existing.to_vec();
    }
    let mut merged = existing.to_vec();
    for flag in JAVA_FLAGS {
        if !merged.iter().any(|argument| key(argument) == key(flag)) {
            merged.push(flag.to_owned());
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chooses_mods_that_exist_for_each_loader() {
        assert!(performance_mods(LoaderKind::Vanilla).is_empty());
        assert!(performance_mods(LoaderKind::Fabric).contains(&"sodium"));
        assert!(performance_mods(LoaderKind::Forge).contains(&"embeddium") && !performance_mods(LoaderKind::Forge).contains(&"sodium"));
        assert!(!performance_mods(LoaderKind::Quilt).contains(&"modernfix"));
        assert!(performance_mods(LoaderKind::NeoForge).contains(&"ferrite-core"));
    }

    #[test]
    fn adds_java_flags_once_and_keeps_the_players_own() {
        let first = with_java_flags(&["-Xss4m".to_owned(), "-XX:MaxGCPauseMillis=50".to_owned()]);
        assert!(first.contains(&"-Xss4m".to_owned()) && first.contains(&"-XX:MaxGCPauseMillis=50".to_owned()));
        assert!(!first.contains(&"-XX:MaxGCPauseMillis=130".to_owned()), "an existing value is never overridden");
        assert!(first.contains(&"-XX:+UseG1GC".to_owned()));
        assert_eq!(with_java_flags(&first), first, "running it twice changes nothing");
    }

    #[test]
    fn respects_a_different_garbage_collector() {
        let expert = vec!["-XX:+UseZGC".to_owned(), "-Xmn512m".to_owned()];
        assert_eq!(with_java_flags(&expert), expert);
    }
}
