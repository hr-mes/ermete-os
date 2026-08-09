use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

/// Struct standard e sicura per la deserializzazione JSON del payload in ingresso.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawServiceIntent {
    pub action: String,
    pub target: String,
    #[serde(default)]
    pub exec_start: Option<String>,
    #[serde(default)]
    pub fallback_exec_start: Option<String>,
    #[serde(default)]
    pub restart_policy: Option<String>,
    #[serde(default)]
    pub environment: HashMap<String, String>,
}

/// Enum che rappresenta l'intento deterministico per un servizio.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ServiceIntent {
    Start {
        target: String,
        exec_start: String,
        fallback_exec_start: Option<String>,
        restart_policy: String,
        environment: HashMap<String, String>,
    },
    Stop {
        target: String,
        exec_start: String,
        fallback_exec_start: Option<String>,
        restart_policy: String,
        environment: HashMap<String, String>,
    },
    Restart {
        target: String,
        exec_start: String,
        fallback_exec_start: Option<String>,
        restart_policy: String,
        environment: HashMap<String, String>,
    },
    Reload {
        target: String,
        exec_start: String,
        fallback_exec_start: Option<String>,
        restart_policy: String,
        environment: HashMap<String, String>,
    },
}

impl ServiceIntent {
    pub fn service_name(&self) -> &str {
        match self {
            ServiceIntent::Start { target, .. }
            | ServiceIntent::Stop { target, .. }
            | ServiceIntent::Restart { target, .. }
            | ServiceIntent::Reload { target, .. } => target,
        }
    }

    pub fn exec_start(&self) -> &str {
        match self {
            ServiceIntent::Start { exec_start, .. }
            | ServiceIntent::Stop { exec_start, .. }
            | ServiceIntent::Restart { exec_start, .. }
            | ServiceIntent::Reload { exec_start, .. } => exec_start,
        }
    }

    pub fn fallback_exec_start(&self) -> Option<&str> {
        match self {
            ServiceIntent::Start { fallback_exec_start, .. }
            | ServiceIntent::Stop { fallback_exec_start, .. }
            | ServiceIntent::Restart { fallback_exec_start, .. }
            | ServiceIntent::Reload { fallback_exec_start, .. } => fallback_exec_start.as_deref(),
        }
    }

    pub fn restart_policy(&self) -> &str {
        match self {
            ServiceIntent::Start { restart_policy, .. }
            | ServiceIntent::Stop { restart_policy, .. }
            | ServiceIntent::Restart { restart_policy, .. }
            | ServiceIntent::Reload { restart_policy, .. } => restart_policy,
        }
    }

    pub fn environment(&self) -> &HashMap<String, String> {
        match self {
            ServiceIntent::Start { environment, .. }
            | ServiceIntent::Stop { environment, .. }
            | ServiceIntent::Restart { environment, .. }
            | ServiceIntent::Reload { environment, .. } => environment,
        }
    }

    pub fn description(&self) -> String {
        format!("Ermete OS Managed Service: {}", self.service_name())
    }
}

pub struct IntentParser;

impl IntentParser {
    pub fn parse(input: &str) -> ServiceIntent {
        let trimmed = input.trim();

        // 1. Prova la deserializzazione diretta nell'enum ServiceIntent
        if let Ok(intent) = serde_json::from_str::<ServiceIntent>(trimmed) {
            info!("Parsed direct JSON enum ServiceIntent -> Target: {}", intent.service_name());
            return intent;
        }

        // 2. Prova la deserializzazione nella struct RawServiceIntent
        let raw: RawServiceIntent = match serde_json::from_str::<RawServiceIntent>(trimmed) {
            Ok(raw_intent) => raw_intent,
            Err(_) => {
                // Fallback deterministico per input non-JSON
                RawServiceIntent {
                    action: "restart".to_string(),
                    target: trimmed.to_lowercase().replace(|c: char| !c.is_alphanumeric() && c != '-', ""),
                    exec_start: None,
                    fallback_exec_start: None,
                    restart_policy: None,
                    environment: HashMap::new(),
                }
            }
        };

        let target = if raw.target.is_empty() {
            "ermete-custom-service".to_string()
        } else {
            raw.target
        };

        let exec_start = raw.exec_start.unwrap_or_else(|| match target.as_str() {
            "nginx" => "/usr/sbin/nginx -g 'daemon off;'".to_string(),
            "redis" => "/usr/bin/redis-server".to_string(),
            "postgresql" | "postgres" => "/usr/bin/postgres".to_string(),
            _ => format!("/usr/bin/{}", target),
        });

        let fallback_exec_start = raw.fallback_exec_start.or_else(|| {
            Some("/usr/bin/python3 -m http.server 8080 --directory /var/www/html".to_string())
        });

        let restart_policy = raw.restart_policy.unwrap_or_else(|| "always".to_string());

        let mut environment = raw.environment;
        environment.entry("ERMETE_INIT_ORACLE_MANAGED".to_string()).or_insert_with(|| "1".to_string());
        environment.entry("AI_AUTONOMOUS_POLICY".to_string()).or_insert_with(|| "strict".to_string());

        let intent = match raw.action.to_lowercase().as_str() {
            "start" => ServiceIntent::Start {
                target,
                exec_start,
                fallback_exec_start,
                restart_policy,
                environment,
            },
            "stop" => ServiceIntent::Stop {
                target,
                exec_start,
                fallback_exec_start,
                restart_policy,
                environment,
            },
            "reload" => ServiceIntent::Reload {
                target,
                exec_start,
                fallback_exec_start,
                restart_policy,
                environment,
            },
            _ => ServiceIntent::Restart {
                target,
                exec_start,
                fallback_exec_start,
                restart_policy,
                environment,
            },
        };

        info!("Parsed JSON Raw Intent -> Action: {:?}, Target: {}", raw.action, intent.service_name());
        intent
    }

    pub fn generate_systemd_unit(intent: &ServiceIntent, use_fallback: bool) -> String {
        let exec = if use_fallback {
            intent
                .fallback_exec_start()
                .unwrap_or("/bin/echo 'Ermete OS Init Oracle Fallback Executed'")
        } else {
            intent.exec_start()
        };

        let mut env_lines = String::new();
        for (k, v) in intent.environment() {
            env_lines.push_str(&format!("Environment=\"{}={}\"\n", k, v));
        }

        let desc_suffix = if use_fallback { " [FALLBACK MODE]" } else { "" };

        format!(
            r#"[Unit]
Description={}{}{}
After=network.target
Wants=network.target

[Service]
Type=simple
ExecStart={}
Restart={}
RestartSec=3s
{}
[Install]
WantedBy=multi-user.target
"#,
            intent.description(),
            desc_suffix,
            if use_fallback { " - Autonomous Recovery Active" } else { "" },
            exec,
            intent.restart_policy(),
            env_lines
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_action_target_intent() {
        let json_input = r#"{"action": "restart", "target": "network"}"#;
        let intent = IntentParser::parse(json_input);

        assert_eq!(intent.service_name(), "network");
        assert_eq!(intent, ServiceIntent::Restart {
            target: "network".to_string(),
            exec_start: "/usr/bin/network".to_string(),
            fallback_exec_start: Some("/usr/bin/python3 -m http.server 8080 --directory /var/www/html".to_string()),
            restart_policy: "always".to_string(),
            environment: HashMap::from([
                ("ERMETE_INIT_ORACLE_MANAGED".to_string(), "1".to_string()),
                ("AI_AUTONOMOUS_POLICY".to_string(), "strict".to_string()),
            ]),
        });

        let unit_content = IntentParser::generate_systemd_unit(&intent, false);
        assert!(unit_content.contains("[Unit]"));
        assert!(unit_content.contains("ExecStart=/usr/bin/network"));
        assert!(unit_content.contains("Restart=always"));
    }

    #[test]
    fn test_parse_full_json_intent() {
        let json_input = r#"{
            "action": "start",
            "target": "custom-redis",
            "exec_start": "/usr/bin/redis-server /etc/redis.conf",
            "fallback_exec_start": "/usr/bin/redis-server",
            "restart_policy": "on-failure",
            "environment": {}
        }"#;

        let intent = IntentParser::parse(json_input);
        assert_eq!(intent.service_name(), "custom-redis");
        assert_eq!(intent.exec_start(), "/usr/bin/redis-server /etc/redis.conf");
        assert_eq!(intent.restart_policy(), "on-failure");
    }

    #[test]
    fn test_generate_fallback_unit() {
        let json_input = r#"{"action": "restart", "target": "nginx"}"#;
        let intent = IntentParser::parse(json_input);
        let fallback_unit = IntentParser::generate_systemd_unit(&intent, true);

        assert!(fallback_unit.contains("FALLBACK MODE"));
        assert!(fallback_unit.contains("http.server"));
    }
}
