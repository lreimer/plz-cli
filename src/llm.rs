use crate::config::{Config, Provider};
use reqwest::blocking::{Client, RequestBuilder};
use serde_json::{Value, json};

pub fn generate(config: &Config, prompt: &str) -> Result<String, String> {
    let response = build_request(&Client::new(), config, prompt)
        .send()
        .map_err(|error| format!("Could not contact {}: {error}", config.provider.name()))?;
    let status = response.status();
    let body = response.json::<Value>().map_err(|_| {
        format!(
            "{} returned an invalid response ({status}).",
            config.provider.name()
        )
    })?;

    if !status.is_success() {
        let message = body["error"]["message"]
            .as_str()
            .unwrap_or("No error details provided");
        return Err(format!(
            "{} API error ({status}): {message}",
            config.provider.name()
        ));
    }

    extract_code(config.provider, &body)
}

fn build_request(client: &Client, config: &Config, prompt: &str) -> RequestBuilder {
    let base = config.api_base.trim_end_matches('/');
    match config.provider {
        Provider::OpenAi => client
            .post(format!("{base}/chat/completions"))
            .bearer_auth(&config.api_key)
            .json(&json!({
                "top_p": 1,
                "temperature": 0,
                "max_completion_tokens": 1000,
                "presence_penalty": 0,
                "frequency_penalty": 0,
                "model": config.model,
                "messages": [{"role": "user", "content": prompt}],
            })),
        Provider::Google => client
            .post(format!("{base}/models/{}:generateContent", config.model))
            .header("x-goog-api-key", &config.api_key)
            .json(&json!({
                "contents": [{"role": "user", "parts": [{"text": prompt}]}],
                "generationConfig": {"temperature": 0},
            })),
    }
}

fn extract_code(provider: Provider, body: &Value) -> Result<String, String> {
    let code = match provider {
        Provider::OpenAi => body["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
        Provider::Google => {
            let candidate = &body["candidates"][0];
            if let Some(reason) = candidate["finishReason"].as_str()
                && reason != "STOP"
            {
                return Err(format!(
                    "Google Gemini did not finish generating code: {reason}"
                ));
            }
            candidate["content"]["parts"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|part| part["thought"].as_bool() != Some(true))
                .filter_map(|part| part["text"].as_str())
                .collect::<String>()
        }
    };
    let code = code.trim();
    if code.is_empty() {
        return Err(format!(
            "{} returned no code. The response may have been blocked.",
            provider.name()
        ));
    }
    Ok(code.to_owned())
}

#[cfg(test)]
mod tests {
    use super::{build_request, extract_code};
    use crate::config::{Config, Provider};
    use reqwest::blocking::Client;
    use serde_json::{Value, json};

    #[test]
    fn builds_provider_specific_requests() -> Result<(), Box<dyn std::error::Error>> {
        for provider in [Provider::OpenAi, Provider::Google] {
            let config = Config {
                provider,
                api_key: "test-key".to_owned(),
                api_base: "http://localhost/v1/".to_owned(),
                model: "test-model".to_owned(),
                shell: String::new(),
            };
            let request = build_request(&Client::new(), &config, "test prompt").build()?;
            let bytes = request
                .body()
                .and_then(reqwest::blocking::Body::as_bytes)
                .ok_or("Missing request body")?;
            let body: Value = serde_json::from_slice(bytes)?;
            match provider {
                Provider::OpenAi => {
                    assert_eq!(request.url().path(), "/v1/chat/completions");
                    assert_eq!(request.headers()["authorization"], "Bearer test-key");
                    assert_eq!(body["model"], "test-model");
                    assert_eq!(body["messages"][0]["content"], "test prompt");
                    assert_eq!(body["max_completion_tokens"], 1000);
                }
                Provider::Google => {
                    assert_eq!(
                        request.url().path(),
                        "/v1/models/test-model:generateContent"
                    );
                    assert_eq!(request.headers()["x-goog-api-key"], "test-key");
                    assert!(!request.headers().contains_key("authorization"));
                    assert!(request.url().query().is_none());
                    assert_eq!(body["contents"][0]["parts"][0]["text"], "test prompt");
                    assert_eq!(body["generationConfig"]["temperature"], 0);
                }
            }
        }
        Ok(())
    }

    #[test]
    fn extracts_openai_code() -> Result<(), String> {
        let body = json!({"choices": [{"message": {"content": "  #!/bin/bash\necho ok\n"}}]});
        assert_eq!(
            extract_code(Provider::OpenAi, &body)?,
            "#!/bin/bash\necho ok"
        );
        Ok(())
    }

    #[test]
    fn joins_gemini_text_parts_without_thoughts() -> Result<(), String> {
        let body = json!({"candidates": [{
            "finishReason": "STOP",
            "content": {"parts": [
                {"text": "internal reasoning", "thought": true},
                {"text": "#!/bin/bash\n"},
                {"text": "echo ok\n"}
            ]}
        }]});
        assert_eq!(
            extract_code(Provider::Google, &body)?,
            "#!/bin/bash\necho ok"
        );
        Ok(())
    }

    #[test]
    fn rejects_missing_empty_and_incomplete_code() {
        for provider in [Provider::OpenAi, Provider::Google] {
            assert!(extract_code(provider, &json!({})).is_err());
        }
        assert!(
            extract_code(
                Provider::OpenAi,
                &json!({
                    "choices": [{"message": {"content": " "}}]
                })
            )
            .is_err()
        );
        for reason in ["SAFETY", "MAX_TOKENS"] {
            assert!(
                extract_code(
                    Provider::Google,
                    &json!({"candidates": [{
                        "finishReason": reason,
                        "content": {"parts": [{"text": "partial code"}]}
                    }]})
                )
                .is_err()
            );
        }
    }
}
