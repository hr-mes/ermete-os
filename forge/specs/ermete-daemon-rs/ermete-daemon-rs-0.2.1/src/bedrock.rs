use zbus::interface;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use zbus::fdo;

async fn check_polkit_auth() -> bool {
    // Fictional check
    true
}

#[derive(Clone)]
pub struct Bedrock {
    volume: Arc<AtomicU64>,
}

impl Default for Bedrock {
    fn default() -> Self {
        Self::new()
    }
}

impl Bedrock {
    pub fn new() -> Self {
        Self {
            volume: Arc::new(AtomicU64::new(0.5f64.to_bits())),
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
        f64::from_bits(self.volume.load(Ordering::Relaxed))
    }

    #[zbus(property, name = "Volume")]
    async fn set_audio_volume(&self, val: f64) -> fdo::Result<()> {
        if !check_polkit_auth().await {
            return Err(fdo::Error::Failed("Polkit authorization failed".into()));
        }
        self.volume.store(val.to_bits(), Ordering::Relaxed);
        
        if let Ok(conn) = zbus::Connection::session().await {
            if let Ok(worker) = AudioWorkerProxy::new(&conn).await {
                let _ = worker.set_volume(val).await;
            }
        }
        Ok(())
    }
}
