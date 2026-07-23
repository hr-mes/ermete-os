use zbus::interface;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Default, Clone)]
pub struct Bedrock {
    volume: Arc<Mutex<f64>>,
}

impl Bedrock {
    pub fn new() -> Self {
        Self {
            volume: Arc::new(Mutex::new(0.5)),
        }
    }
}

#[interface(name = "os.ermete.Bedrock")]
impl Bedrock {
    async fn ping(&self) -> String {
        "pong".to_string()
    }

    #[zbus(property, name = "Volume")]
    async fn audio_volume(&self) -> f64 {
        *self.volume.lock().await
    }

    #[zbus(property, name = "Volume")]
    async fn set_audio_volume(&self, val: f64) {
        let mut vol = self.volume.lock().await;
        *vol = val;
        let vol_str = format!("{:.2}", val);
        let _ = tokio::process::Command::new("wpctl")
            .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &vol_str])
            .output()
            .await;
    }
}
