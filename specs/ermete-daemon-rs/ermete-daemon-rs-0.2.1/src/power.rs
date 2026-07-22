use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{sleep, Duration};
use zbus::Connection;

#[zbus::proxy(
    interface = "org.freedesktop.UPower",
    default_service = "org.freedesktop.UPower",
    default_path = "/org/freedesktop/UPower"
)]
trait UPower {
    #[zbus(property)]
    fn on_battery(&self) -> zbus::Result<bool>;
}

#[zbus::proxy(
    interface = "net.hadess.PowerProfiles",
    default_service = "net.hadess.PowerProfiles",
    default_path = "/net/hadess/PowerProfiles"
)]
trait PowerProfiles {
    #[zbus(property)]
    fn active_profile(&self) -> zbus::Result<String>;
    
    #[zbus(property)]
    fn set_active_profile(&self, profile: &str) -> zbus::Result<()>;
}

pub struct PowerManager {
    pub on_battery: Arc<AtomicBool>,
}

impl PowerManager {
    pub fn new() -> Self {
        Self {
            on_battery: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn start_monitoring(&self, sys_conn: Connection) {
        let on_battery_state = self.on_battery.clone();
        tokio::spawn(async move {
            let upower = match UPowerProxy::new(&sys_conn).await {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("PowerManager: failed to create UPower proxy: {}", e);
                    return;
                }
            };
            let profiles = match PowerProfilesProxy::new(&sys_conn).await {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("PowerManager: failed to create PowerProfiles proxy: {}", e);
                    return;
                }
            };

            let mut last_battery_state = false;

            loop {
                if let Ok(on_battery) = upower.on_battery().await {
                    on_battery_state.store(on_battery, Ordering::Relaxed);
                    
                    if on_battery != last_battery_state {
                        last_battery_state = on_battery;
                        if on_battery {
                            println!("PowerManager: System is on battery. Switching to powersave profile.");
                            let _ = profiles.set_active_profile("power-saver").await;
                        } else {
                            println!("PowerManager: System is plugged in. Switching to balanced profile.");
                            let _ = profiles.set_active_profile("balanced").await;
                        }
                    }
                }

                sleep(Duration::from_secs(10)).await;
            }
        });
    }
}
