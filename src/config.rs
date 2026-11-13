use std::{env, io::Write, process::exit};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    OpenAi,
    Google,
}

impl Provider {
    fn parse(value: Option<&str>) -> Result<Self, String> {
        match value
            .unwrap_or("openai")
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "" | "openai" => Ok(Self::OpenAi),
            "google" | "gemini" => Ok(Self::Google),
            value => Err(format!(
                "Unsupported PLZ_PROVIDER: {value}. Choose openai or google."
            )),
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::OpenAi => "OpenAI",
            Self::Google => "Google Gemini",
        }
    }
}

pub struct Config {
    pub provider: Provider,
    pub api_key: String,
    pub api_base: String,
    pub model: String,
    pub shell: String,
}

impl Config {
    pub fn new() -> Result<Self, String> {
        Self::from_env(|name| env::var(name).ok())
    }

    fn from_env(mut get: impl FnMut(&str) -> Option<String>) -> Result<Self, String> {
        let provider = Provider::parse(get("PLZ_PROVIDER").as_deref())?;
        let (key_var, base_var, model_var, default_base, default_model) = match provider {
            Provider::OpenAi => (
                "OPENAI_API_KEY",
                "OPENAI_API_BASE",
                "OPENAI_MODEL",
                "https://api.openai.com/v1",
                "gpt-5.4-mini",
            ),
            Provider::Google => (
                "GEMINI_API_KEY",
                "GEMINI_API_BASE",
                "GEMINI_MODEL",
                "https://generativelanguage.googleapis.com/v1beta",
                "gemini-2.5-flash",
            ),
        };
        let api_key = get(key_var)
            .filter(|key| !key.trim().is_empty())
            .or_else(|| {
                (provider == Provider::Google)
                    .then(|| get("GOOGLE_API_KEY"))
                    .flatten()
                    .filter(|key| !key.trim().is_empty())
            })
            .ok_or_else(|| {
                format!(
                    "{} requires an API key. Please set {key_var}.",
                    provider.name()
                )
            })?;
        let api_base = resolve_setting(get(base_var).as_deref(), default_base);
        let model = resolve_setting(get(model_var).as_deref(), default_model);
        let shell = get("SHELL").unwrap_or_default();

        Ok(Self {
            provider,
            api_key,
            api_base,
            model,
            shell,
        })
    }

    pub fn write_to_history(&self, code: &str) {
        let history_file = match self.shell.as_str() {
            "/bin/bash" => std::env::var("HOME").unwrap() + "/.bash_history",
            "/bin/zsh" => std::env::var("HOME").unwrap() + "/.zsh_history",
            _ => return,
        };

        std::fs::OpenOptions::new()
            .append(true)
            .open(history_file)
            .map_or((), |mut file| {
                file.write_all(format!("{code}\n").as_bytes())
                    .unwrap_or_else(|_| {
                        exit(1);
                    });
            });
    }
}

fn resolve_setting(value: Option<&str>, default: &str) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(default)
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::{Config, Provider, resolve_setting};

    #[test]
    fn uses_configured_model() {
        assert_eq!(
            resolve_setting(Some(" custom-model "), "gpt-5.4-mini"),
            "custom-model"
        );
    }

    #[test]
    fn defaults_when_model_is_unset() {
        assert_eq!(resolve_setting(None, "gpt-5.4-mini"), "gpt-5.4-mini");
    }

    #[test]
    fn defaults_when_model_is_blank() {
        for model in ["", " ", "\t\n"] {
            assert_eq!(resolve_setting(Some(model), "gpt-5.4-mini"), "gpt-5.4-mini");
        }
    }

    fn config(values: &[(&str, &str)]) -> Result<Config, String> {
        Config::from_env(|name| {
            values
                .iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| (*value).to_owned())
        })
    }

    #[test]
    fn preserves_openai_defaults() -> Result<(), String> {
        let config = config(&[("OPENAI_API_KEY", "test-key")])?;
        assert_eq!(config.provider, Provider::OpenAi);
        assert_eq!(config.model, "gpt-5.4-mini");
        assert_eq!(config.api_base, "https://api.openai.com/v1");
        Ok(())
    }

    #[test]
    fn google_does_not_require_openai_credentials() -> Result<(), String> {
        let config = config(&[("PLZ_PROVIDER", " Google "), ("GEMINI_API_KEY", "test-key")])?;
        assert_eq!(config.provider, Provider::Google);
        assert_eq!(config.model, "gemini-2.5-flash");
        assert_eq!(
            config.api_base,
            "https://generativelanguage.googleapis.com/v1beta"
        );
        Ok(())
    }

    #[test]
    fn uses_google_settings_and_key_fallback() -> Result<(), String> {
        let config = config(&[
            ("PLZ_PROVIDER", "gemini"),
            ("GOOGLE_API_KEY", "google-key"),
            ("GEMINI_MODEL", " custom-gemini "),
            ("GEMINI_API_BASE", " http://localhost:8080/v1beta "),
            ("OPENAI_MODEL", "ignored"),
        ])?;
        assert_eq!(config.api_key, "google-key");
        assert_eq!(config.model, "custom-gemini");
        assert_eq!(config.api_base, "http://localhost:8080/v1beta");
        Ok(())
    }

    #[test]
    fn rejects_unknown_provider_and_missing_selected_key() {
        assert!(config(&[("PLZ_PROVIDER", "unknown")]).is_err());
        let result = config(&[
            ("PLZ_PROVIDER", "google"),
            ("OPENAI_API_KEY", "wrong-provider-key"),
            ("GEMINI_API_KEY", " "),
        ]);
        assert!(matches!(result, Err(message) if message.contains("GEMINI_API_KEY")));
    }
}
