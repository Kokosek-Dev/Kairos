#[derive(Debug, Clone)]
pub struct Endpoints {
    pub authorize: String,
    pub token: String,
    pub xbox_user: String,
    pub xsts: String,
    pub minecraft_login: String,
    pub entitlements: String,
    pub profile: String,
}

impl Default for Endpoints {
    fn default() -> Self {
        Self {
            authorize: "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize".into(),
            token: "https://login.microsoftonline.com/consumers/oauth2/v2.0/token".into(),
            xbox_user: "https://user.auth.xboxlive.com/user/authenticate".into(),
            xsts: "https://xsts.auth.xboxlive.com/xsts/authorize".into(),
            minecraft_login: "https://api.minecraftservices.com/authentication/login_with_xbox".into(),
            entitlements: "https://api.minecraftservices.com/entitlements/mcstore".into(),
            profile: "https://api.minecraftservices.com/minecraft/profile".into(),
        }
    }
}

#[cfg(test)]
impl Endpoints {
    pub fn with_base(base: &str) -> Self {
        Self {
            authorize: format!("{base}/authorize"),
            token: format!("{base}/token"),
            xbox_user: format!("{base}/xbl"),
            xsts: format!("{base}/xsts"),
            minecraft_login: format!("{base}/mc/login"),
            entitlements: format!("{base}/mc/entitlements"),
            profile: format!("{base}/mc/profile"),
        }
    }
}
