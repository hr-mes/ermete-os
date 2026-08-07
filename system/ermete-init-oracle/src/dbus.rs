use crate::intent::IntentParser;
use crate::systemd_manager::SystemdManager;
use zbus::interface;

pub struct InitOracleInterface {
    manager: SystemdManager,
}

impl InitOracleInterface {
    pub fn new(manager: SystemdManager) -> Self {
        Self { manager }
    }
}

#[interface(name = "org.ermete.InitOracle")]
impl InitOracleInterface {
    async fn submit_intent(&self, intent_text: String) -> String {
        let intent = IntentParser::parse(&intent_text);
        match self.manager.apply_intent(intent).await {
            Ok(record) => serde_json::to_string_pretty(&record).unwrap_or_else(|_| "{}".to_string()),
            Err(e) => {
                serde_json::json!({
                    "error": e.to_string(),
                    "status": "failed"
                })
                .to_string()
            }
        }
    }

    async fn status(&self) -> String {
        let services = self.manager.list_services().await;
        serde_json::json!({
            "daemon": "ermete-init-oracle",
            "version": "1.0.0",
            "systemd_target_dir": self.manager.get_target_dir(),
            "active_managed_services_count": services.len(),
            "status": "ONLINE"
        })
        .to_string()
    }

    async fn list_managed_services(&self) -> String {
        let services = self.manager.list_services().await;
        serde_json::to_string_pretty(&services).unwrap_or_else(|_| "[]".to_string())
    }

    async fn get_service_status(&self, service_name: String) -> String {
        let unit_name = format!("{}.service", service_name);
        let status = self.manager.check_service_status(&unit_name).await;
        serde_json::json!({
            "service_name": service_name,
            "unit_name": unit_name,
            "systemd_status": status
        })
        .to_string()
    }

    async fn revert_service(&self, service_name: String) -> String {
        match self.manager.revert_service(&service_name).await {
            Ok(msg) => serde_json::json!({ "success": true, "message": msg }).to_string(),
            Err(e) => serde_json::json!({ "success": false, "error": e.to_string() }).to_string(),
        }
    }
}
