use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::{path::PathBuf, time::Duration};

#[derive(Clone, Deserialize, Serialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub ai: Ai,
    pub general: General,
    pub greeting: Greeting,
    pub preset_prompts: Vec<Preset>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Ai {
    pub provider: String,
    #[serde(skip_serializing)]
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}
impl Default for Ai {
    fn default() -> Self {
        Self {
            provider: "openai".into(),
            api_key: String::new(),
            model: "gpt-4.1-mini".into(),
            base_url: "https://api.openai.com/v1".into(),
        }
    }
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct General {
    pub clipboard_auto_paste: bool,
    pub auto_copy_result: bool,
    pub default_n: usize,
}
impl Default for General {
    fn default() -> Self {
        Self {
            clipboard_auto_paste: false,
            auto_copy_result: false,
            default_n: 3,
        }
    }
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Greeting {
    pub enabled: bool,
    pub opening: String,
    pub closing: String,
}
impl Default for Greeting {
    fn default() -> Self {
        Self {
            enabled: true,
            opening: "相手に合った自然な挨拶を添える".into(),
            closing: "丁寧な結びの言葉を添える".into(),
        }
    }
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Preset {
    pub name: String,
    pub prompt: String,
}
pub fn presets() -> Vec<Preset> {
    [
        ("丁寧に", "礼儀正しく、落ち着いた丁寧な文体で返信する。"),
        ("カジュアルに", "親しみやすく自然な文体で返信する。"),
        ("簡潔に", "要点を絞って短く明確に返信する。"),
        (
            "熱意を伝える",
            "前向きな気持ちと熱意が伝わる文体で返信する。",
        ),
    ]
    .into_iter()
    .map(|(name, prompt)| Preset {
        name: name.into(),
        prompt: prompt.into(),
    })
    .collect()
}
pub fn directory() -> Result<PathBuf, String> {
    dirs::home_dir()
        .map(|p| p.join(".config/toneweave"))
        .ok_or_else(|| "Cannot locate your home directory.".into())
}
pub fn merge(base: &mut Value, over: Value) {
    match (base, over) {
        (Value::Mapping(base), Value::Mapping(over)) => {
            for (key, value) in over {
                merge(base.entry(key).or_insert(Value::Null), value);
            }
        }
        (base, over) => *base = over,
    }
}
fn parse_mapping(text: &str) -> Result<Value, String> {
    let value: Value = serde_yaml::from_str(text)
        .map_err(|_| "Invalid config YAML. Check syntax and indentation.".to_string())?;
    if !value.is_mapping() {
        return Err("Config YAML must be a mapping.".into());
    }
    Ok(value)
}
async fn override_yaml(command: &str) -> Result<Value, String> {
    let args = shlex::split(command).ok_or("Unbalanced quotes in config_override_command.")?;
    if args.is_empty() {
        return Err("config_override_command must not be empty.".into());
    }
    let mut path = std::env::var("PATH").unwrap_or_default();
    for extra in ["/opt/homebrew/bin", "/usr/local/bin"] {
        if !path.split(':').any(|p| p == extra) {
            path.push(':');
            path.push_str(extra);
        }
    }
    let output = tokio::time::timeout(
        Duration::from_secs(30),
        tokio::process::Command::new(&args[0])
            .args(&args[1..])
            .env("PATH", path)
            .stdin(std::process::Stdio::null())
            .kill_on_drop(true)
            .output(),
    )
    .await
    .map_err(|_| "Config override timed out after 30 seconds.")?
    .map_err(|_| "Could not start config_override_command.")?;
    if !output.status.success() {
        return Err("config_override_command failed. Check the command in your terminal.".into());
    }
    let text =
        String::from_utf8(output.stdout).map_err(|_| "Config override must return UTF-8 YAML.")?;
    parse_mapping(&text)
}
pub async fn load_path(path: &std::path::Path) -> Result<Config, String> {
    let mut value = match std::fs::read_to_string(path) {
        Ok(text) => parse_mapping(&text)?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Value::Mapping(Default::default()),
        Err(_) => return Err("Could not read ~/.config/toneweave/config.yaml.".into()),
    };
    let command = value
        .as_mapping_mut()
        .unwrap()
        .remove(Value::String("config_override_command".into()));
    if let Some(command) = command {
        let command = command
            .as_str()
            .ok_or("config_override_command must be a string.")?;
        merge(&mut value, override_yaml(command).await?);
        value
            .as_mapping_mut()
            .unwrap()
            .remove(Value::String("config_override_command".into()));
    }
    let mut config: Config = serde_yaml::from_value(value).map_err(|_| {
        "Invalid config fields or types. Compare with config.example.yaml.".to_string()
    })?;
    if config.preset_prompts.is_empty() {
        config.preset_prompts = presets();
    }
    if config.ai.api_key.trim().is_empty() {
        config.ai.api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
    }
    config.validate()?;
    Ok(config)
}
pub async fn load() -> Result<Config, String> {
    load_path(&directory()?.join("config.yaml")).await
}
impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if !["openai", "openai-compatible"].contains(&self.ai.provider.as_str()) {
            return Err("ai.provider must be openai or openai-compatible.".into());
        }
        if !(1..=3).contains(&self.general.default_n) {
            return Err("general.default_n must be between 1 and 3.".into());
        }
        if self.ai.model.trim().is_empty() {
            return Err("ai.model must not be empty.".into());
        }
        let url = reqwest::Url::parse(&self.ai.base_url)
            .map_err(|_| "ai.base_url must be a valid URL.")?;
        let local = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
        if !(url.scheme() == "https" || url.scheme() == "http" && local)
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err("ai.base_url must use HTTPS (HTTP is allowed for localhost), without credentials, query or fragment.".into());
        }
        let mut names = std::collections::HashSet::new();
        for preset in &self.preset_prompts {
            if preset.name.trim().is_empty()
                || preset.prompt.trim().is_empty()
                || !names.insert(&preset.name)
            {
                return Err(
                    "Preset names must be unique and names/prompts must not be empty.".into(),
                );
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recursive_merge_replaces_lists() {
        let mut base =
            parse_mapping("ai: {model: old, provider: openai}\npreset_prompts: [a, b]").unwrap();
        merge(
            &mut base,
            parse_mapping("ai: {model: new}\npreset_prompts: [c]").unwrap(),
        );
        assert_eq!(base["ai"]["provider"].as_str(), Some("openai"));
        assert_eq!(base["ai"]["model"].as_str(), Some("new"));
        assert_eq!(base["preset_prompts"].as_sequence().unwrap().len(), 1);
    }
    #[tokio::test]
    async fn override_is_not_shell() {
        let v = override_yaml("/usr/bin/printf 'value: \"$(echo secret) | cat\"'")
            .await
            .unwrap();
        assert_eq!(v["value"].as_str(), Some("$(echo secret) | cat"));
        assert!(override_yaml("/bin/echo '").await.is_err());
        assert!(override_yaml("/usr/bin/false").await.is_err());
    }
    #[test]
    fn secrets_are_never_serialized() {
        let mut c = Config::default();
        c.ai.api_key = "secret-sentinel".into();
        assert!(!serde_json::to_string(&c)
            .unwrap()
            .contains("secret-sentinel"));
        c.general.default_n = 0;
        assert!(c.validate().is_err());
    }
}
