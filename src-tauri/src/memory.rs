use serde::Serialize;

const STEP_MB: u64 = 256;
const MIN_MB: u64 = 1024;
const KEEP_FREE_MB: u64 = 1024;

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryAdvice {
    pub total_mb: u64,
    pub recommended_mb: u64,
    /// The most a game should be given, so the rest of the computer keeps working.
    pub max_mb: u64,
}

fn round_down(value: u64) -> u64 {
    value / STEP_MB * STEP_MB
}

/// Suggests how much memory the game should get. Modded games need noticeably more than the plain one.
pub fn advise(total_mb: u64, modded: bool) -> MemoryAdvice {
    let max_mb = round_down(total_mb.saturating_sub(KEEP_FREE_MB)).max(MIN_MB);
    let wanted = match (total_mb, modded) {
        (..=4096, false) => 1536,
        (..=4096, true) => 2048,
        (..=8192, false) => 2048,
        (..=8192, true) => 3072,
        (..=16384, false) => 3072,
        (..=16384, true) => 4096,
        (_, false) => 4096,
        (_, true) => 6144,
    };
    MemoryAdvice { total_mb, recommended_mb: wanted.clamp(MIN_MB, max_mb), max_mb }
}

pub fn total_mb() -> u64 {
    let mut system = sysinfo::System::new();
    system.refresh_memory();
    system.total_memory() / (1024 * 1024)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recommends_more_for_modded_games_and_for_bigger_computers() {
        assert_eq!(advise(16384, false).recommended_mb, 3072);
        assert_eq!(advise(16384, true).recommended_mb, 4096);
        assert_eq!(advise(32768, true).recommended_mb, 6144);
        assert_eq!(advise(8192, true).recommended_mb, 3072);
        assert_eq!(advise(4096, false).recommended_mb, 1536);
    }

    #[test]
    fn never_asks_for_more_than_the_computer_can_spare() {
        let tiny = advise(2048, true);
        assert_eq!((tiny.recommended_mb, tiny.max_mb), (1024, 1024));
        let small = advise(3072, true);
        assert_eq!((small.recommended_mb, small.max_mb), (2048, 2048));
        assert_eq!(advise(16384, true).max_mb, 15360);
    }

    #[test]
    fn reads_the_memory_of_this_machine() {
        assert!(total_mb() >= 512);
    }
}
