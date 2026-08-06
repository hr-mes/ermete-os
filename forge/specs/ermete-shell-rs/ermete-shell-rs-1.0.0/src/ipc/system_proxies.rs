use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

pub use crate::ipc::types::*;

pub trait ControllerBackend: Any + Send + Sync {
    fn name(&self) -> &'static str;
    fn as_any(&self) -> &dyn Any;
}

static GLOBAL_REGISTRY: OnceLock<ProxyRegistry> = OnceLock::new();
static GLOBAL_EVENT_BUS: OnceLock<SystemEventBus> = OnceLock::new();

pub fn get_event_bus() -> SystemEventBus {
    GLOBAL_EVENT_BUS
        .get_or_init(SystemEventBus::new)
        .clone()
}

pub fn subscribe_system_events() -> tokio::sync::broadcast::Receiver<SystemEvent> {
    get_event_bus().subscribe_broadcast()
}

#[allow(dead_code)]
pub fn emit_system_event(event: SystemEvent) {
    get_event_bus().emit(event);
}

pub fn get_registry() -> &'static ProxyRegistry {
    GLOBAL_REGISTRY.get_or_init(|| ProxyRegistry::new(get_event_bus()))
}

pub struct ProxyRegistry {
    controllers: Mutex<HashMap<&'static str, Arc<dyn ControllerBackend>>>,
    event_bus: SystemEventBus,
}

impl ProxyRegistry {
    pub fn new(event_bus: SystemEventBus) -> Self {
        Self {
            controllers: Mutex::new(HashMap::new()),
            event_bus,
        }
    }

    pub fn register(&self, controller: Box<dyn ControllerBackend>) {
        let name = controller.name();
        let arc_controller: Arc<dyn ControllerBackend> = Arc::from(controller);
        if let Ok(mut map) = self.controllers.lock() {
            map.insert(name, arc_controller);
        }
    }

    #[allow(dead_code)]
    pub fn register_arc(&self, controller: Arc<dyn ControllerBackend>) {
        let name = controller.name();
        if let Ok(mut map) = self.controllers.lock() {
            map.insert(name, controller);
        }
    }

    #[allow(dead_code)]
    pub fn get(&self, name: &str) -> Option<Arc<dyn ControllerBackend>> {
        if let Ok(map) = self.controllers.lock() {
            map.get(name).cloned()
        } else {
            None
        }
    }

    pub fn get_typed<T: 'static + Clone>(&self, name: &str) -> Option<T> {
        if let Ok(map) = self.controllers.lock() {
            if let Some(ctrl) = map.get(name) {
                if let Some(concrete) = ctrl.as_any().downcast_ref::<T>() {
                    return Some(concrete.clone());
                }
            }
        }
        None
    }

    #[allow(dead_code)]
    pub fn emit_event(&self, event: SystemEvent) {
        self.event_bus.emit(event);
    }

    #[allow(dead_code)]
    pub fn event_bus(&self) -> &SystemEventBus {
        &self.event_bus
    }
}

pub fn init_system_controller(controllers: Vec<Box<dyn ControllerBackend>>) {
    let registry = get_registry();
    for controller in controllers {
        registry.register(controller);
    }
}
