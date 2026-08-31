use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;

pub use crate::ipc::types::*;

pub trait ControllerBackend: Any + Send + Sync {
    fn name(&self) -> &'static str;
    fn as_any(&self) -> &dyn Any;
}

static GLOBAL_REGISTRY: OnceLock<ProxyRegistry> = OnceLock::new();
static GLOBAL_AUDIO_BUS: OnceLock<AudioBus> = OnceLock::new();
static GLOBAL_NET_BUS: OnceLock<NetBus> = OnceLock::new();
static GLOBAL_HARDWARE_BUS: OnceLock<HardwareBus> = OnceLock::new();
static GLOBAL_MPRIS_BUS: OnceLock<MprisBus> = OnceLock::new();

pub fn get_audio_bus() -> AudioBus { GLOBAL_AUDIO_BUS.get_or_init(AudioBus::new).clone() }
pub fn get_net_bus() -> NetBus { GLOBAL_NET_BUS.get_or_init(NetBus::new).clone() }
pub fn get_hardware_bus() -> HardwareBus { GLOBAL_HARDWARE_BUS.get_or_init(HardwareBus::new).clone() }
pub fn get_mpris_bus() -> MprisBus { GLOBAL_MPRIS_BUS.get_or_init(MprisBus::new).clone() }





pub fn get_registry() -> &'static ProxyRegistry { GLOBAL_REGISTRY.get_or_init(|| ProxyRegistry::new()) }

pub struct ProxyRegistry {
    controllers: Mutex<HashMap<&'static str, Arc<dyn ControllerBackend>>>,
    
}

impl ProxyRegistry {
    pub fn new() -> Self {
        Self {
            controllers: Mutex::new(HashMap::new()),
            
        }
    }

    pub fn register(&self, controller: Box<dyn ControllerBackend>) {
        let name = controller.name();
        let arc_controller: Arc<dyn ControllerBackend> = Arc::from(controller);
        let mut map = self.controllers.blocking_lock();
        {
            map.insert(name, arc_controller);
        }
    }

    #[allow(dead_code)]
    pub fn register_arc(&self, controller: Arc<dyn ControllerBackend>) {
        let name = controller.name();
        let mut map = self.controllers.blocking_lock();
        {
            map.insert(name, controller);
        }
    }

    #[allow(dead_code)]
    pub fn get(&self, name: &str) -> Option<Arc<dyn ControllerBackend>> {
        let map = self.controllers.blocking_lock();
        map.get(name).cloned()
    }

    pub fn get_typed<T: 'static + Clone>(&self, name: &str) -> Option<T> {
        let map = self.controllers.blocking_lock();
        {
            if let Some(ctrl) = map.get(name) {
                if let Some(concrete) = ctrl.as_any().downcast_ref::<T>() {
                    return Some(concrete.clone());
                }
            }
        }
        None
    }

    
    }

pub fn init_system_controller(controllers: Vec<Box<dyn ControllerBackend>>) {
    let registry = get_registry();
    for controller in controllers {
        registry.register(controller);
    }
}


