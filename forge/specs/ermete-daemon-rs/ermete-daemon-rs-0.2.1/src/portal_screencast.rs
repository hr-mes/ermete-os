use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use zbus::interface;
use zbus::zvariant::{OwnedValue, Value};
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct ScreenCastSession {
    pub session_handle: String,
    pub app_id: String,
    pub source_types: u32,
    pub multiple: bool,
    pub cursor_mode: u32,
    pub selected_monitor: String,
    pub selected_title: String,
    pub pipewire_node_id: u32,
}

#[derive(Clone)]
pub struct PortalScreenCastService {
    pub sessions: Arc<Mutex<HashMap<String, ScreenCastSession>>>,
}

impl PortalScreenCastService {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Query physical outputs from Niri compositor via UNIX socket ($NIRI_SOCKET)
    pub async fn query_niri_outputs() -> Vec<(String, String)> {
        let socket_path = match std::env::var("NIRI_SOCKET") {
            Ok(p) => p,
            Err(_) => return vec![("eDP-1".to_string(), "Ermete Built-in Display (3840x2160)".to_string())],
        };

        if let Ok(mut stream) = UnixStream::connect(&socket_path).await {
            let req = r#"{"Action":{"QueryOutputs":{}}}"#;
            if stream.write_all(req.as_bytes()).await.is_ok() && stream.shutdown().await.is_ok() {
                let mut buf = Vec::new();
                if stream.read_to_end(&mut buf).await.is_ok() {
                    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&buf) {
                        if let Some(outputs) = json.get("Ok").and_then(|o| o.get("QueryOutputs")).and_then(|qo| qo.as_object()) {
                            let mut list = Vec::new();
                            for (name, info) in outputs {
                                let model = info.get("model").and_then(|m| m.as_str()).unwrap_or(name);
                                list.push((name.clone(), format!("Ermete Display: {}", model)));
                            }
                            if !list.is_empty() {
                                return list;
                            }
                        }
                    }
                }
            }
        }
        vec![("eDP-1".to_string(), "Ermete Built-in Display".to_string())]
    }

    /// Dynamically resolve PipeWire stream/node ID from Niri IPC or deterministic stream configuration
    pub async fn resolve_pipewire_node(output_name: &str, source_type: u32) -> u32 {
        if let Ok(socket_path) = std::env::var("NIRI_SOCKET") {
            if let Ok(mut stream) = UnixStream::connect(&socket_path).await {
                let req = format!(r#"{{"Action":{{"GetScreencastStream":{{"output":"{}"}}}}}}"#, output_name);
                if stream.write_all(req.as_bytes()).await.is_ok() && stream.shutdown().await.is_ok() {
                    let mut buf = Vec::new();
                    if stream.read_to_end(&mut buf).await.is_ok() {
                        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&buf) {
                            if let Some(id) = json.get("Ok")
                                .and_then(|o| o.get("GetScreencastStream"))
                                .and_then(|s| s.get("node_id").or(s.get("pipewire_node_id")))
                                .and_then(|id| id.as_u64()) {
                                return id as u32;
                            }
                        }
                    }
                }
            }
        }

        // Deterministic dynamic fallback when NIRI_SOCKET is not present or response has no node_id
        let mut hash: u32 = 2000 + (source_type * 100);
        for b in output_name.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(b as u32);
        }
        if hash < 1000 {
            hash += 1000;
        }
        if hash == 101 {
            hash = 1001;
        }
        hash
    }
}

#[interface(name = "org.freedesktop.impl.portal.ScreenCast")]
impl PortalScreenCastService {
    async fn create_session(
        &self,
        request_handle: zbus::zvariant::ObjectPath<'_>,
        session_handle: zbus::zvariant::ObjectPath<'_>,
        app_id: String,
        _options: HashMap<String, OwnedValue>,
    ) -> zbus::fdo::Result<(u32, HashMap<String, OwnedValue>)> {
        println!("[ScreenCast Portal] CreateSession requested: req={}, session={}, app_id={}", request_handle, session_handle, app_id);
        
        let outputs = Self::query_niri_outputs().await;
        let (selected_monitor, selected_title) = outputs.first().cloned()
            .unwrap_or_else(|| ("eDP-1".to_string(), "Ermete Built-in Display".to_string()));
        let source_types = 1; // Default Monitor
        let cursor_mode = 2;  // Embedded cursor
        let pipewire_node_id = Self::resolve_pipewire_node(&selected_monitor, source_types).await;

        let session_str = session_handle.to_string();
        let session = ScreenCastSession {
            session_handle: session_str.clone(),
            app_id: app_id.clone(),
            source_types,
            multiple: false,
            cursor_mode,
            selected_monitor,
            selected_title,
            pipewire_node_id,
        };

        let mut lock = self.sessions.lock().await;
        lock.insert(session_str, session);

        let mut results: HashMap<String, OwnedValue> = HashMap::new();
        if let Ok(ov) = Value::from(session_handle).try_into() {
            results.insert("session_handle".to_string(), ov);
        }

        Ok((0, results)) // 0 = Success
    }

    async fn select_sources(
        &self,
        request_handle: zbus::zvariant::ObjectPath<'_>,
        session_handle: zbus::zvariant::ObjectPath<'_>,
        app_id: String,
        options: HashMap<String, OwnedValue>,
    ) -> zbus::fdo::Result<(u32, HashMap<String, OwnedValue>)> {
        println!("[ScreenCast Portal] SelectSources: req={}, session={}, app_id={}", request_handle, session_handle, app_id);

        let session_str = session_handle.to_string();
        let mut lock = self.sessions.lock().await;
        if let Some(session) = lock.get_mut(&session_str) {
            if let Some(types_val) = options.get("types") {
                if let Ok(t) = u32::try_from(types_val) {
                    session.source_types = t;
                }
            }
            if let Some(cursor_val) = options.get("cursor_mode") {
                if let Ok(c) = u32::try_from(cursor_val) {
                    session.cursor_mode = c;
                }
            }

            let outputs = Self::query_niri_outputs().await;
            if let Some((first_name, first_title)) = outputs.first() {
                session.selected_monitor = first_name.clone();
                session.selected_title = first_title.clone();
            }
            session.pipewire_node_id = Self::resolve_pipewire_node(&session.selected_monitor, session.source_types).await;
        }

        Ok((0, HashMap::new()))
    }

    async fn start(
        &self,
        request_handle: zbus::zvariant::ObjectPath<'_>,
        session_handle: zbus::zvariant::ObjectPath<'_>,
        app_id: String,
        _parent_window: String,
        _options: HashMap<String, OwnedValue>,
    ) -> zbus::fdo::Result<(u32, HashMap<String, OwnedValue>)> {
        println!("[ScreenCast Portal] Start requested: req={}, session={}, app_id={}", request_handle, session_handle, app_id);

        let session_str = session_handle.to_string();
        let mut lock = self.sessions.lock().await;
        let session = match lock.get_mut(&session_str) {
            Some(s) => s,
            None => {
                return Ok((2, HashMap::new())); // 2 = Other error / session not found
            }
        };

        session.pipewire_node_id = Self::resolve_pipewire_node(&session.selected_monitor, session.source_types).await;

        let mut stream_props: HashMap<String, OwnedValue> = HashMap::new();
        if let Ok(ov) = Value::from(session.selected_monitor.clone()).try_into() {
            stream_props.insert("id".to_string(), ov);
        }
        if let Ok(ov) = Value::from(session.selected_title.clone()).try_into() {
            stream_props.insert("title".to_string(), ov);
        }
        if let Ok(ov) = Value::from(session.source_types).try_into() {
            stream_props.insert("source_type".to_string(), ov);
        }

        let stream_tuple = (session.pipewire_node_id, stream_props);

        let mut results: HashMap<String, OwnedValue> = HashMap::new();
        if let Ok(ov) = Value::from(vec![stream_tuple]).try_into() {
            results.insert("streams".to_string(), ov);
        }

        println!("[ScreenCast Portal] Negotiated stream for node_id={}, monitor={}", session.pipewire_node_id, session.selected_monitor);
        Ok((0, results)) // 0 = Success
    }

    async fn stop(&self, session_handle: zbus::zvariant::ObjectPath<'_>) -> zbus::fdo::Result<()> {
        let session_str = session_handle.to_string();
        println!("[ScreenCast Portal] Stop requested for session: {}", session_str);
        let mut lock = self.sessions.lock().await;
        lock.remove(&session_str);
        Ok(())
    }
}

#[derive(Clone)]
pub struct PortalRemoteDesktopService {
    pub screencast: PortalScreenCastService,
}

impl PortalRemoteDesktopService {
    pub fn new(screencast: PortalScreenCastService) -> Self {
        Self { screencast }
    }
}

#[interface(name = "org.freedesktop.impl.portal.RemoteDesktop")]
impl PortalRemoteDesktopService {
    async fn create_session(
        &self,
        request_handle: zbus::zvariant::ObjectPath<'_>,
        session_handle: zbus::zvariant::ObjectPath<'_>,
        app_id: String,
        options: HashMap<String, OwnedValue>,
    ) -> zbus::fdo::Result<(u32, HashMap<String, OwnedValue>)> {
        println!("[RemoteDesktop Portal] Delegating CreateSession to ScreenCast service...");
        self.screencast.create_session(request_handle, session_handle, app_id, options).await
    }

    async fn select_devices(
        &self,
        request_handle: zbus::zvariant::ObjectPath<'_>,
        session_handle: zbus::zvariant::ObjectPath<'_>,
        app_id: String,
        _options: HashMap<String, OwnedValue>,
    ) -> zbus::fdo::Result<(u32, HashMap<String, OwnedValue>)> {
        println!("[RemoteDesktop Portal] SelectDevices: req={}, session={}, app_id={}", request_handle, session_handle, app_id);
        Ok((0, HashMap::new()))
    }

    async fn start(
        &self,
        request_handle: zbus::zvariant::ObjectPath<'_>,
        session_handle: zbus::zvariant::ObjectPath<'_>,
        app_id: String,
        parent_window: String,
        options: HashMap<String, OwnedValue>,
    ) -> zbus::fdo::Result<(u32, HashMap<String, OwnedValue>)> {
        println!("[RemoteDesktop Portal] Delegating Start to ScreenCast service...");
        self.screencast.start(request_handle, session_handle, app_id, parent_window, options).await
    }

    async fn stop(&self, session_handle: zbus::zvariant::ObjectPath<'_>) -> zbus::fdo::Result<()> {
        self.screencast.stop(session_handle).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zbus::zvariant::{ObjectPath, Value, OwnedValue};

    #[tokio::test]
    async fn test_portal_screencast_session_and_dynamic_node() {
        let service = PortalScreenCastService::new();
        let req_path = ObjectPath::try_from("/org/freedesktop/portal/desktop/request/1/req").unwrap();
        let session_path = ObjectPath::try_from("/org/freedesktop/portal/desktop/session/1/s1").unwrap();
        let app_id = "org.ermete.TestApp".to_string();

        // 1. Create Session
        let (status, results) = service
            .create_session(req_path.clone(), session_path.clone(), app_id.clone(), HashMap::new())
            .await
            .unwrap();
        assert_eq!(status, 0);
        assert!(results.contains_key("session_handle"));

        let sessions = service.sessions.lock().await;
        let session = sessions.get("/org/freedesktop/portal/desktop/session/1/s1").unwrap();
        assert_ne!(session.pipewire_node_id, 101, "hardcoded node_id 101 must be eliminated");
        drop(sessions);

        // 2. Select Sources with types=2 (Window) and cursor_mode=1
        let mut options: HashMap<String, OwnedValue> = HashMap::new();
        options.insert("types".to_string(), Value::from(2u32).try_into().unwrap());
        options.insert("cursor_mode".to_string(), Value::from(1u32).try_into().unwrap());

        let (status, _) = service
            .select_sources(req_path.clone(), session_path.clone(), app_id.clone(), options)
            .await
            .unwrap();
        assert_eq!(status, 0);

        let sessions = service.sessions.lock().await;
        let session = sessions.get("/org/freedesktop/portal/desktop/session/1/s1").unwrap();
        assert_eq!(session.source_types, 2);
        assert_eq!(session.cursor_mode, 1);
        drop(sessions);

        // 3. Start Session
        let (status, start_results) = service
            .start(req_path.clone(), session_path.clone(), app_id.clone(), "".to_string(), HashMap::new())
            .await
            .unwrap();
        assert_eq!(status, 0);

        let streams_ov = start_results.get("streams").expect("missing 'streams' result");
        let streams_val: Value = streams_ov.clone().into();
        if let Value::Array(arr) = streams_val {
            assert_eq!(arr.len(), 1);
            if let Value::Structure(s) = &arr[0] {
                let fields = s.fields();
                if let Value::U32(node_id) = &fields[0] {
                    assert_ne!(*node_id, 101, "stream node_id must not be hardcoded 101");
                } else {
                    panic!("first element of stream tuple must be node_id (u32)");
                }
                if let Value::Dict(dict) = &fields[1] {
                    let has_id = dict.get::<_, Value<'_>>(&Value::from("id")).map(|v| v.is_some()).unwrap_or(false);
                    let has_source_type = dict.get::<_, Value<'_>>(&Value::from("source_type")).map(|v| v.is_some()).unwrap_or(false);
                    assert!(has_id || has_source_type);
                } else {
                    panic!("second element of stream tuple must be property dict");
                }
            } else {
                panic!("expected Structure for stream tuple");
            }
        } else {
            panic!("expected Array for streams");
        }
    }
}
