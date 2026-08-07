use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceIntent {
    pub service_name: String,
    pub description: String,
    pub exec_start: String,
    pub fallback_exec_start: Option<String>,
    pub restart_policy: String,
    pub environment: HashMap<String, String>,
}

pub struct IntentParser;

impl IntentParser {
    pub fn parse(input: &str) -> ServiceIntent {
        let trimmed = input.trim();

        // 1. Try parsing JSON first if input is JSON formatted
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            if let Ok(intent) = serde_json::from_str::<ServiceIntent>(trimmed) {
                return intent;
            }
        }

        // 2. Otherwise parse natural language prompt (e.g. AI intent)
        let lower = trimmed.to_lowercase();
        
        let service_name = if lower.contains("nginx") {
            "nginx".to_string()
        } else if lower.contains("redis") {
            "redis".to_string()
        } else if lower.contains("postgres") || lower.contains("postgresql") {
            "postgresql".to_string()
        } else if lower.contains("docker") {
            "docker".to_string()
        } else if lower.contains("http") || lower.contains("web") {
            "web-server".to_string()
        } else {
            // Extract first meaningful word or fallback
            trimmed
                .split_whitespace()
                .find(|w| w.len() > 3 && !["assicurati", "che", "server", "servizio", "riavvialo", "attivo", "della"].contains(&w.to_lowercase().as_str()))
                .unwrap_or("ermete-custom-service")
                .to_lowercase()
                .replace(|c: char| !c.is_alphanumeric() && c != '-', "")
        };

        let exec_start = if lower.contains("nginx") {
            "/usr/sbin/nginx -g 'daemon off;'".to_string()
        } else if lower.contains("redis") {
            "/usr/bin/redis-server".to_string()
        } else if lower.contains("postgres") {
            "/usr/bin/postgres".to_string()
        } else if lower.contains("python") || lower.contains("web") {
            "/usr/bin/python3 -m http.server 8080".to_string()
        } else {
            format!("/usr/bin/{}", service_name)
        };

        // Fallback executable if primary binary is not found or fails
        let fallback_exec_start = Some(format!(
            "/usr/bin/python3 -m http.server 8080 --directory /var/www/html"
        ));

        let restart_policy = if lower.contains("riavvialo") || lower.contains("restart") || lower.contains("cade") {
            "always".to_string()
        } else {
            "on-failure".to_string()
        };

        let mut environment = HashMap::new();
        environment.insert("ERMETE_INIT_ORACLE_MANAGED".to_string(), "1".to_string());
        environment.insert("AI_AUTONOMOUS_POLICY".to_string(), "strict".to_string());

        let intent = ServiceIntent {
            service_name: service_name.clone(),
            description: format!("Ermete OS Autonomous AI Managed Service: {}", service_name),
            exec_start,
            fallback_exec_start,
            restart_policy,
            environment,
        };

        info!("Parsed AI Intent -> Service: {}, Exec: {}", intent.service_name, intent.exec_start);
        intent
    }

    pub fn generate_systemd_unit(intent: &ServiceIntent, use_fallback: bool) -> String {
        let exec = if use_fallback {
            intent
                .fallback_exec_start
                .as_deref()
                .unwrap_or("/bin/echo 'Ermete OS Init Oracle Fallback Executed'")
        } else {
            &intent.exec_start
        };

        let mut env_lines = String::new();
        for (k, v) in &intent.environment {
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
            intent.description,
            desc_suffix,
            if use_fallback { " - Autonomous Recovery Active" } else { "" },
            exec,
            intent.restart_policy,
            env_lines
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_natural_language_intent() {
        let input = "assicurati che il server Nginx sia attivo e riavvialo se cade";
        let intent = IntentParser::parse(input);

        assert_eq!(intent.service_name, "nginx");
        assert!(intent.exec_start.contains("nginx"));
        assert_eq!(intent.restart_policy, "always");

        let unit_content = IntentParser::generate_systemd_unit(&intent, false);
        assert!(unit_content.contains("[Unit]"));
        assert!(unit_content.contains("ExecStart=/usr/sbin/nginx"));
        assert!(unit_content.contains("Restart=always"));
    }

    #[test]
    fn test_parse_json_intent() {
        let json_input = r#"{
            "service_name": "custom-redis",
            "description": "Custom Redis Service",
            "exec_start": "/usr/bin/redis-server /etc/redis.conf",
            "fallback_exec_start": "/usr/bin/redis-server",
            "restart_policy": "on-failure",
            "environment": {}
        }"#;

        let intent = IntentParser::parse(json_input);
        assert_eq!(intent.service_name, "custom-redis");
        assert_eq!(intent.exec_start, "/usr/bin/redis-server /etc/redis.conf");
        assert_eq!(intent.restart_policy, "on-failure");
    }

    #[test]
    fn test_generate_fallback_unit() {
        let input = "assicurati che il server Nginx sia attivo";
        let intent = IntentParser::parse(input);
        let fallback_unit = IntentParser::generate_systemd_unit(&intent, true);

        assert!(fallback_unit.contains("FALLBACK MODE"));
        assert!(fallback_unit.contains("http.server"));
    }
}

