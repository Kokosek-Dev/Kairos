use crate::instances::model::{Instance, LoaderKind};
use discord_rich_presence::activity::{Activity, Assets, Timestamps};
use discord_rich_presence::{DiscordIpc, DiscordIpcClient};
use std::sync::Mutex;

const APPLICATION_ID_ENV: &str = "KAIROS_DISCORD_APP_ID";
const BUILT_IN_APPLICATION_ID: Option<&str> = option_env!("KAIROS_DISCORD_APP_ID");
/// The name of the picture uploaded under Rich Presence, Art Assets in the Discord application.
const LARGE_IMAGE: &str = "kairos";

fn is_application_id(value: &str) -> bool {
    (17..=20).contains(&value.len()) && value.bytes().all(|byte| byte.is_ascii_digit())
}

/// The Discord application that "Playing Kairos" belongs to. Without one, the presence stays off.
fn application_id() -> Option<String> {
    std::env::var(APPLICATION_ID_ENV).ok().or_else(|| BUILT_IN_APPLICATION_ID.map(str::to_owned)).filter(|id| is_application_id(id))
}

fn loader_name(loader: LoaderKind) -> Option<&'static str> {
    match loader {
        LoaderKind::Vanilla => None,
        LoaderKind::Fabric => Some("Fabric"),
        LoaderKind::Quilt => Some("Quilt"),
        LoaderKind::Forge => Some("Forge"),
        LoaderKind::NeoForge => Some("NeoForge"),
    }
}

/// The two lines shown on the player's Discord profile: the instance, and what it runs.
pub fn describe(instance: &Instance, mods: usize) -> (String, String) {
    let mut parts = vec![format!("Minecraft {}", instance.game_version)];
    if let Some(loader) = loader_name(instance.loader) {
        parts.push(loader.to_owned());
    }
    if mods > 0 {
        parts.push(format!("{mods} mods"));
    }
    (instance.name.clone(), parts.join(" · "))
}

/// Tells Discord what the player is playing. Everything here is best effort: no Discord, no problem.
pub struct Presence {
    application_id: Option<String>,
    client: Mutex<Option<DiscordIpcClient>>,
}

impl Presence {
    pub fn new() -> Self {
        Self { application_id: application_id(), client: Mutex::new(None) }
    }

    pub fn is_available(&self) -> bool {
        self.application_id.is_some()
    }

    pub fn show(&self, details: &str, state: &str, started_at: i64) {
        let Some(id) = &self.application_id else { return };
        let mut slot = self.client.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if slot.is_none() {
            let mut client = DiscordIpcClient::new(id);
            if client.connect().is_err() {
                return;
            }
            *slot = Some(client);
        }
        let activity = Activity::new()
            .details(details)
            .state(state)
            .timestamps(Timestamps::new().start(started_at))
            .assets(Assets::new().large_image(LARGE_IMAGE).large_text("Kairos"));
        if slot.as_mut().is_some_and(|client| client.set_activity(activity).is_err()) {
            *slot = None;
        }
    }

    pub fn clear(&self) {
        let mut slot = self.client.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(mut client) = slot.take() {
            let _ = client.clear_activity();
            let _ = client.close();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instances::model::InstanceIcon;
    use chrono::Utc;

    fn instance(loader: LoaderKind) -> Instance {
        Instance {
            schema_version: 1,
            id: "demo".into(),
            name: "Survival".into(),
            icon: InstanceIcon::Preset { key: "cube".into() },
            game_version: "1.21.1".into(),
            loader,
            loader_version: None,
            memory_mb: 4096,
            jvm_args: Vec::new(),
            java_path: None,
            created_at: Utc::now(),
            last_played_at: None,
            play_time_secs: 0,
        }
    }

    #[test]
    fn describes_what_is_being_played() {
        assert_eq!(describe(&instance(LoaderKind::Vanilla), 0), ("Survival".to_owned(), "Minecraft 1.21.1".to_owned()));
        assert_eq!(describe(&instance(LoaderKind::Fabric), 34).1, "Minecraft 1.21.1 · Fabric · 34 mods");
    }

    #[test]
    fn only_real_discord_application_ids_switch_it_on() {
        assert!(is_application_id("1234567890123456789"));
        assert!(!is_application_id("") && !is_application_id("abc") && !is_application_id("12345"));
        assert!(!is_application_id("12345678901234567890123"));
    }

    #[test]
    fn stays_quiet_without_an_application() {
        let presence = Presence { application_id: None, client: Mutex::new(None) };
        presence.show("a", "b", 0);
        presence.clear();
        assert!(!presence.is_available());
    }
}
