use zbus::interface;
use std::sync::Arc;
use tokio::sync::Mutex;
use zbus::fdo;

async fn check_polkit_auth() -> bool {
    // Fictional check
    true
}

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

#[zbus::proxy(
    interface = "os.ermete.AudioWorker",
    default_service = "os.ermete.AudioWorker",
    default_path = "/os/ermete/AudioWorker"
)]
trait AudioWorker {
    fn set_volume(&self, volume: f64) -> zbus::Result<()>;
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
    async fn set_audio_volume(&self, val: f64) -> fdo::Result<()> {
        if !check_polkit_auth().await {
            return Err(fdo::Error::Failed("Polkit authorization failed".into()));
        }
        let mut vol = self.volume.lock().await;
        *vol = val;
        
        if let Ok(conn) = zbus::Connection::session().await {
            if let Ok(worker) = AudioWorkerProxy::new(&conn).await {
                let _ = worker.set_volume(val).await;
            }
        }
        Ok(())
    }
}
