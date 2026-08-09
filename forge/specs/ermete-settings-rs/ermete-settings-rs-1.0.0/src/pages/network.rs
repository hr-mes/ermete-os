#![allow(deprecated)]
#![allow(unsafe_code)]
use anyhow::{anyhow, Context, Result};
use gtk4::prelude::*;
use gtk4::{Align, Box, Button, DropDown, Entry, Label, ListBox, Orientation};
use libc::{
    c_void, ftruncate, mmap, munmap, shm_open, shm_unlink, MAP_SHARED, O_CREAT, O_RDWR,
    PROT_READ, PROT_WRITE,
};
use std::ffi::CString;
use std::os::unix::io::RawFd;
use std::ptr::{self, NonNull};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use crate::components::action_row::ActionRow;

pub const RING_BUFFER_MAGIC: u64 = 0x4552_4D54_5348_4D31;
pub const FLAG_ACTIVE: u32 = 0x0001;

// Unikernel Ring Buffer Frame Types
pub const FRAME_TELEMETRY: u16 = 0x0001;
pub const FRAME_CHECK_CONNECTIVITY: u16 = 0x0101;
pub const FRAME_SCAN_NETWORKS: u16 = 0x0102;
pub const FRAME_CONNECT_WIFI: u16 = 0x0103;
pub const FRAME_ADD_VPN: u16 = 0x0104;

pub const FRAME_STATUS_CONNECTIVITY: u16 = 0x0201;
pub const FRAME_STATUS_NETWORKS: u16 = 0x0202;
pub const FRAME_STATUS_WIFI_RESULT: u16 = 0x0203;
pub const FRAME_STATUS_VPN_RESULT: u16 = 0x0204;

#[repr(C)]
pub struct RingBufferHeader {
    pub magic: u64,
    pub capacity: usize,
    pub head: AtomicUsize,
    _pad_head: [u8; 56],
    pub tail: AtomicUsize,
    _pad_tail: [u8; 56],
    pub flags: AtomicU32,
    _pad_flags: [u8; 60],
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameHeader {
    pub payload_len: u32,
    pub frame_type: u16,
    pub flags: u16,
}

pub struct ZeroCopyRingBuffer {
    fd: RawFd,
    ptr: NonNull<u8>,
    total_size: usize,
    capacity: usize,
    is_owner: bool,
    shm_name: Option<String>,
}

unsafe impl Send for ZeroCopyRingBuffer {}
unsafe impl Sync for ZeroCopyRingBuffer {}

impl ZeroCopyRingBuffer {
    pub fn header_size() -> usize {
        std::mem::size_of::<RingBufferHeader>()
    }

    pub fn open_or_create(name: &str, capacity: usize) -> Result<Self> {
        let formatted_name = if name.starts_with('/') {
            name.to_string()
        } else {
            format!("/{}", name)
        };
        let c_name = anyhow::Context::context(CString::new(formatted_name.clone()), "Invalid POSIX shm name")?;

        let fd = unsafe { shm_open(c_name.as_ptr(), O_RDWR, 0o660) };
        if fd >= 0 {
            Self::init_from_fd(fd, capacity, false, Some(formatted_name))
        } else {
            let fd_create = unsafe { shm_open(c_name.as_ptr(), O_CREAT | O_RDWR, 0o660) };
            if fd_create < 0 {
                let c_memfd = anyhow::Context::context(CString::new(name), "Invalid memfd name")?;
                let fd_mem = unsafe { libc::memfd_create(c_memfd.as_ptr(), libc::MFD_CLOEXEC) };
                if fd_mem < 0 {
                    return Err(anyhow::Error::from(std::io::Error::last_os_error()))
                        .context("Failed to allocate shared memory or memfd ring buffer");
                }
                Self::init_from_fd(fd_mem, capacity, true, None)
            } else {
                Self::init_from_fd(fd_create, capacity, true, Some(formatted_name))
            }
        }
    }

    fn init_from_fd(
        fd: RawFd,
        capacity: usize,
        is_owner: bool,
        shm_name: Option<String>,
    ) -> Result<Self> {
        let total_size = Self::header_size() + capacity;
        let trunc_res = unsafe { ftruncate(fd, total_size as libc::off_t) };
        if trunc_res < 0 && is_owner {
            unsafe { libc::close(fd) };
            return Err(anyhow::Error::from(std::io::Error::last_os_error())).context("ftruncate failed");
        }
        let mapped = unsafe {
            mmap(
                ptr::null_mut(),
                total_size,
                PROT_READ | PROT_WRITE,
                MAP_SHARED,
                fd,
                0,
            )
        };
        if mapped == libc::MAP_FAILED {
            unsafe { libc::close(fd) };
            return Err(anyhow::Error::from(std::io::Error::last_os_error())).context("mmap failed");
        }
        let ptr = NonNull::new(mapped as *mut u8)
            .ok_or_else(|| anyhow!("mmap returned null pointer"))?;

        if is_owner {
            unsafe {
                let header = ptr.as_ptr() as *mut RingBufferHeader;
                ptr::write_bytes(header, 0, 1);
                (*header).magic = RING_BUFFER_MAGIC;
                (*header).capacity = capacity;
                (*header).head.store(0, Ordering::Relaxed);
                (*header).tail.store(0, Ordering::Relaxed);
                (*header).flags.store(FLAG_ACTIVE, Ordering::Relaxed);
            }
        }

        Ok(Self {
            fd,
            ptr,
            total_size,
            capacity,
            is_owner,
            shm_name,
        })
    }

    #[inline]
    fn header(&self) -> &RingBufferHeader {
        unsafe { &*(self.ptr.as_ptr() as *const RingBufferHeader) }
    }

    #[inline]
    fn data_ptr(&self) -> *mut u8 {
        unsafe { self.ptr.as_ptr().add(Self::header_size()) }
    }

    pub fn available_write(&self) -> usize {
        let head = self.header().head.load(Ordering::Relaxed);
        let tail = self.header().tail.load(Ordering::Acquire);
        let occupied = head.wrapping_sub(tail);
        self.capacity.saturating_sub(occupied)
    }

    pub fn available_read(&self) -> usize {
        let head = self.header().head.load(Ordering::Acquire);
        let tail = self.header().tail.load(Ordering::Relaxed);
        head.wrapping_sub(tail)
    }

    pub fn push(&self, data: &[u8]) -> Result<usize> {
        let len = data.len();
        if len == 0 {
            return Ok(0);
        }
        let avail = self.available_write();
        if avail < len {
            return Err(anyhow!("Ring buffer full (requested {} bytes, available {} bytes)", len, avail));
        }

        let head = self.header().head.load(Ordering::Relaxed);
        let write_offset = head % self.capacity;
        let data_ptr = self.data_ptr();

        let first_chunk = usize::min(len, self.capacity - write_offset);
        let second_chunk = len - first_chunk;

        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), data_ptr.add(write_offset), first_chunk);
            if second_chunk > 0 {
                ptr::copy_nonoverlapping(data.as_ptr().add(first_chunk), data_ptr, second_chunk);
            }
        }

        self.header().head.fetch_add(len, Ordering::Release);
        Ok(len)
    }

    pub fn pop(&self, buf: &mut [u8]) -> Result<usize> {
        let max_len = buf.len();
        if max_len == 0 {
            return Ok(0);
        }
        let avail = self.available_read();
        if avail == 0 {
            return Ok(0);
        }
        let read_len = usize::min(max_len, avail);
        let tail = self.header().tail.load(Ordering::Relaxed);
        let read_offset = tail % self.capacity;
        let data_ptr = self.data_ptr();

        let first_chunk = usize::min(read_len, self.capacity - read_offset);
        let second_chunk = read_len - first_chunk;

        unsafe {
            ptr::copy_nonoverlapping(data_ptr.add(read_offset), buf.as_mut_ptr(), first_chunk);
            if second_chunk > 0 {
                ptr::copy_nonoverlapping(data_ptr, buf.as_mut_ptr().add(first_chunk), second_chunk);
            }
        }

        self.header().tail.fetch_add(read_len, Ordering::Release);
        Ok(read_len)
    }

    pub fn push_frame(&self, frame_type: u16, data: &[u8]) -> Result<usize> {
        let frame_header = FrameHeader {
            payload_len: data.len() as u32,
            frame_type,
            flags: 0,
        };
        let header_bytes = unsafe {
            std::slice::from_raw_parts(
                &frame_header as *const FrameHeader as *const u8,
                std::mem::size_of::<FrameHeader>(),
            )
        };
        let total_frame_len = header_bytes.len() + data.len();
        let avail = self.available_write();
        if avail < total_frame_len {
            return Err(anyhow!("Cannot push frame: insufficient space"));
        }
        self.push(header_bytes)?;
        self.push(data)?;
        Ok(total_frame_len)
    }

    pub fn pop_frame(&self) -> Result<Option<(u16, Vec<u8>)>> {
        let header_size = std::mem::size_of::<FrameHeader>();
        let avail = self.available_read();
        if avail < header_size {
            return Ok(None);
        }

        let mut header_buf = [0u8; std::mem::size_of::<FrameHeader>()];
        let tail = self.header().tail.load(Ordering::Relaxed);
        let read_offset = tail % self.capacity;
        let data_ptr = self.data_ptr();

        let first_chunk = usize::min(header_size, self.capacity - read_offset);
        let second_chunk = header_size - first_chunk;

        unsafe {
            ptr::copy_nonoverlapping(data_ptr.add(read_offset), header_buf.as_mut_ptr(), first_chunk);
            if second_chunk > 0 {
                ptr::copy_nonoverlapping(data_ptr, header_buf.as_mut_ptr().add(first_chunk), second_chunk);
            }
        }

        let frame_header: FrameHeader =
            unsafe { std::ptr::read_unaligned(header_buf.as_ptr() as *const FrameHeader) };
        let total_needed = header_size + frame_header.payload_len as usize;
        if avail < total_needed {
            return Ok(None);
        }

        self.header().tail.fetch_add(header_size, Ordering::Release);
        let mut payload = vec![0u8; frame_header.payload_len as usize];
        if frame_header.payload_len > 0 {
            self.pop(&mut payload)?;
        }
        Ok(Some((frame_header.frame_type, payload)))
    }
}

impl Drop for ZeroCopyRingBuffer {
    fn drop(&mut self) {
        unsafe {
            munmap(self.ptr.as_ptr() as *mut c_void, self.total_size);
            libc::close(self.fd);
            if self.is_owner {
                if let Some(ref name) = self.shm_name {
                    if let Ok(c_name) = CString::new(name.as_str()) {
                        shm_unlink(c_name.as_ptr());
                    }
                }
            }
        }
    }
}

/// Zero-Trust Unikernel Ring Buffer Client for UI Event Submission & Passive Status Reading
#[derive(Clone)]
pub struct UnikernelNetworkConsumer {
    tx_ring: Arc<Mutex<Option<ZeroCopyRingBuffer>>>,
    rx_ring: Arc<Mutex<Option<ZeroCopyRingBuffer>>>,
}

impl UnikernelNetworkConsumer {
    pub fn new() -> Self {
        let tx = ZeroCopyRingBuffer::open_or_create("ermete-net-ui-rx", 2 * 1024 * 1024).ok();
        let rx = ZeroCopyRingBuffer::open_or_create("ermete-net-ui-tx", 2 * 1024 * 1024).ok();
        Self {
            tx_ring: Arc::new(Mutex::new(tx)),
            rx_ring: Arc::new(Mutex::new(rx)),
        }
    }

    /// Submits an asynchronous event on the ZeroCopyRingBuffer without waiting for synchronous RPC response
    pub fn submit_event(&self, frame_type: u16, payload: &[u8]) -> Result<()> {
        if let Ok(guard) = self.tx_ring.lock() {
            if let Some(ref ring) = *guard {
                ring.push_frame(frame_type, payload)?;
                return Ok(());
            }
        }
        Err(anyhow!("ZeroCopyRingBuffer TX channel unavailable"))
    }

    /// Reads next return status event from return Ring Buffer
    pub fn poll_passive_status(&self) -> Result<Option<(u16, Vec<u8>)>> {
        if let Ok(guard) = self.rx_ring.lock() {
            if let Some(ref ring) = *guard {
                return ring.pop_frame();
            }
        }
        Ok(None)
    }
}

pub fn build_page() -> Box {
    let consumer = UnikernelNetworkConsumer::new();

    let container = Box::new(Orientation::Vertical, 20);
    container.set_margin_top(24);
    container.set_margin_bottom(32);
    container.set_margin_start(24);
    container.set_margin_end(24);

    // Title
    let title = Label::new(Some("Rete, Wi-Fi Aziendale & VPN (Unikernel Ring Buffer)"));
    title.add_css_class("title-1");
    title.set_halign(Align::Start);
    container.append(&title);

    // Connectivity Card
    let check_btn = Button::with_label("Aggiorna Stato");
    let conn_status_subtitle = Label::new(Some("In attesa di eventi passivi da Unikernel..."));
    conn_status_subtitle.set_halign(Align::Start);
    conn_status_subtitle.add_css_class("action-row-subtitle");

    let conn_title = Label::new(Some("Stato Connettività"));
    conn_title.set_halign(Align::Start);
    conn_title.add_css_class("action-row-title");

    let conn_text_box = Box::new(Orientation::Vertical, 4);
    conn_text_box.set_hexpand(true);
    conn_text_box.append(&conn_title);
    conn_text_box.append(&conn_status_subtitle);

    let conn_row = Box::new(Orientation::Horizontal, 12);
    conn_row.add_css_class("action-row");
    conn_row.append(&conn_text_box);
    conn_row.append(&check_btn);

    let consumer_check = consumer.clone();
    check_btn.connect_clicked(move |_| {
        let _ = consumer_check.submit_event(FRAME_CHECK_CONNECTIVITY, &[]);
    });
    container.append(&conn_row);

    // --- Standard Wi-Fi Scan Section ---
    let wifi_title = Label::new(Some("Reti Wi-Fi Disponibili"));
    wifi_title.add_css_class("title-2");
    wifi_title.set_halign(Align::Start);
    wifi_title.set_margin_top(12);
    container.append(&wifi_title);

    let scan_btn = Button::with_label("Scansiona Reti");
    scan_btn.set_halign(Align::Start);

    let wifi_scan_row = ActionRow::builder("Scansione Wi-Fi")
        .subtitle("Invio evento ScanNetworks via ZeroCopyRingBuffer")
        .suffix(&scan_btn)
        .build();
    container.append(&wifi_scan_row);

    let list_box = ListBox::new();
    list_box.add_css_class("boxed-list");
    container.append(&list_box);

    let consumer_scan = consumer.clone();
    let list_box_clone = list_box.clone();
    scan_btn.connect_clicked(move |_| {
        let list_box = list_box_clone.clone();
        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }
        let loading_row = ActionRow::builder("Evento sottomesso su Ring Buffer...")
            .subtitle("Scansione asincrona in corso nel Unikernel SmolTCP")
            .build();
        list_box.append(&loading_row);

        let _ = consumer_scan.submit_event(FRAME_SCAN_NETWORKS, &[]);
    });

    // --- Enterprise Wi-Fi 802.1x Section ---
    let ent_title = Label::new(Some("Configurazione Wi-Fi Aziendale (802.1x EAP-TLS / PEAP)"));
    ent_title.add_css_class("title-2");
    ent_title.set_halign(Align::Start);
    ent_title.set_margin_top(16);
    container.append(&ent_title);

    let ent_box = Box::new(Orientation::Vertical, 8);
    ent_box.add_css_class("card");

    let ent_ssid = Entry::builder().placeholder_text("es. Azienda-Corp").build();
    let ent_id = Entry::builder().placeholder_text("es. mario.rossi@azienda.it").build();
    let ent_pwd = Entry::builder().placeholder_text("Password o PIN Token").visibility(false).build();
    let ent_eap = DropDown::from_strings(&["PEAP (MSCHAPv2)", "EAP-TLS (Certificato)", "TTLS"]);
    let ent_ca = Entry::builder().placeholder_text("/etc/pki/tls/cert.pem").build();

    let row_ssid = ActionRow::builder("Nome Rete (SSID)")
        .subtitle("Identificativo SSID aziendale")
        .suffix(&ent_ssid)
        .build();
    let row_id = ActionRow::builder("Identità")
        .subtitle("Utente o nome certificato")
        .suffix(&ent_id)
        .build();
    let row_pwd = ActionRow::builder("Password")
        .subtitle("Credenziale di accesso")
        .suffix(&ent_pwd)
        .build();
    let row_eap = ActionRow::builder("Metodo EAP")
        .subtitle("Seleziona protocollo di autenticazione 802.1x")
        .suffix(&ent_eap)
        .build();
    let row_ca = ActionRow::builder("Certificato CA")
        .subtitle("Percorso del certificato CA di sistema")
        .suffix(&ent_ca)
        .build();

    ent_box.append(&row_ssid);
    ent_box.append(&row_id);
    ent_box.append(&row_pwd);
    ent_box.append(&row_eap);
    ent_box.append(&row_ca);

    let ent_btn = Button::with_label("Attiva Profilo 802.1x Aziendale");
    ent_btn.add_css_class("suggested-action");
    ent_btn.set_halign(Align::Start);

    let ent_status = Label::new(None);
    ent_status.set_halign(Align::Start);

    let row_ent_action = ActionRow::builder("Attivazione 802.1x")
        .subtitle("Sottomette configurazione su Ring Buffer Zero-Trust")
        .suffix(&ent_btn)
        .build();
    ent_box.append(&row_ent_action);

    container.append(&ent_box);
    container.append(&ent_status);

    let consumer_ent = consumer.clone();
    let ent_status_clone = ent_status.clone();
    ent_btn.connect_clicked(move |_| {
        let ssid = ent_ssid.text().to_string();
        let id = ent_id.text().to_string();
        let pwd = ent_pwd.text().to_string();
        let eap = match ent_eap.selected() {
            1 => "tls".to_string(),
            2 => "ttls".to_string(),
            _ => "peap".to_string(),
        };
        let ca = ent_ca.text().to_string();
        let payload = format!("{},{},{},{},{}", ssid, id, pwd, eap, ca);

        match consumer_ent.submit_event(FRAME_CONNECT_WIFI, payload.as_bytes()) {
            Ok(_) => ent_status_clone.set_text("⚡ Evento ConnectToWifi sottomesso su ZeroCopyRingBuffer"),
            Err(e) => ent_status_clone.set_text(&format!("❌ Errore sottomissione event: {:?}", e)),
        }
    });

    // --- VPN Section ---
    let vpn_title = Label::new(Some("Tunnel VPN Nativi (WireGuard & OpenVPN)"));
    vpn_title.add_css_class("title-2");
    vpn_title.set_halign(Align::Start);
    vpn_title.set_margin_top(16);
    container.append(&vpn_title);

    let vpn_box = Box::new(Orientation::Vertical, 8);
    vpn_box.add_css_class("card");

    let vpn_name = Entry::builder().placeholder_text("es. Azienda-WG").build();
    let vpn_type = DropDown::from_strings(&["WireGuard (wg-quick)", "OpenVPN"]);
    let vpn_path = Entry::builder().placeholder_text("Percorso .conf o .ovpn").build();

    let row_vpn_name = ActionRow::builder("Nome Tunnel")
        .subtitle("Nome identificativo della VPN")
        .suffix(&vpn_name)
        .build();
    let row_vpn_type = ActionRow::builder("Tipo VPN")
        .subtitle("Tecnologia del tunnel")
        .suffix(&vpn_type)
        .build();
    let row_vpn_path = ActionRow::builder("File Configurazione")
        .subtitle("Percorso assoluto del file di configurazione")
        .suffix(&vpn_path)
        .build();

    vpn_box.append(&row_vpn_name);
    vpn_box.append(&row_vpn_type);
    vpn_box.append(&row_vpn_path);

    let vpn_btn = Button::with_label("Aggiungi e Connetti VPN");
    vpn_btn.add_css_class("suggested-action");
    vpn_btn.set_halign(Align::Start);

    let row_vpn_action = ActionRow::builder("Configura VPN")
        .subtitle("Sottomette parametri tunnel al Unikernel")
        .suffix(&vpn_btn)
        .build();
    vpn_box.append(&row_vpn_action);

    container.append(&vpn_box);

    let vpn_status = Label::new(None);
    vpn_status.set_halign(Align::Start);
    container.append(&vpn_status);

    let consumer_vpn = consumer.clone();
    let vpn_status_clone = vpn_status.clone();
    vpn_btn.connect_clicked(move |_| {
        let name = vpn_name.text().to_string();
        let v_type = if vpn_type.selected() == 1 { "openvpn" } else { "wireguard" };
        let path = vpn_path.text().to_string();
        let payload = format!("{},{},{}", name, v_type, path);

        match consumer_vpn.submit_event(FRAME_ADD_VPN, payload.as_bytes()) {
            Ok(_) => vpn_status_clone.set_text("⚡ Evento AddVpnTunnel sottomesso su ZeroCopyRingBuffer"),
            Err(e) => vpn_status_clone.set_text(&format!("❌ Errore sottomissione event: {:?}", e)),
        }
    });

    // --- Passive Event Poller (Reads Return Ring Buffer) ---
    let consumer_poller = consumer.clone();
    let list_box_poller = list_box.clone();
    let ent_status_poller = ent_status.clone();
    let vpn_status_poller = vpn_status.clone();
    let conn_status_poller = conn_status_subtitle.clone();

    relm4::spawn_local(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(50));
        loop {
            interval.tick().await;
            while let Ok(Some((frame_type, payload))) = consumer_poller.poll_passive_status() {
                match frame_type {
                    FRAME_STATUS_CONNECTIVITY => {
                        let text = String::from_utf8_lossy(&payload);
                        let sub = match text.as_ref() {
                            "FULL" => "🌐 Connesso (Accesso Completo via SmolTCP Unikernel)",
                            "PORTAL" => "⚠️ Captive Portal Rilevato",
                            "LIMITED" => "⚠️ Connessione Limitata",
                            "NONE" => "❌ Nessuna Connessione",
                            other => other,
                        };
                        conn_status_poller.set_text(sub);
                    }
                    FRAME_STATUS_NETWORKS => {
                        while let Some(child) = list_box_poller.first_child() {
                            list_box_poller.remove(&child);
                        }
                        let text = String::from_utf8_lossy(&payload);
                        if text.is_empty() {
                            let empty_row = ActionRow::builder("Nessuna rete trovata")
                                .subtitle("Assicurati che l'interfaccia Wi-Fi sia attiva")
                                .build();
                            list_box_poller.append(&empty_row);
                        } else {
                            for ssid in text.split(',') {
                                if !ssid.is_empty() {
                                    let connect_net_btn = Button::with_label("Connetti");
                                    let row = ActionRow::builder(ssid)
                                        .subtitle("Rete Wi-Fi Rilevata (Ring Buffer Passive Return)")
                                        .suffix(&connect_net_btn)
                                        .build();
                                    list_box_poller.append(&row);
                                }
                            }
                        }
                    }
                    FRAME_STATUS_WIFI_RESULT => {
                        let text = String::from_utf8_lossy(&payload);
                        ent_status_poller.set_text(&format!("✅ Stato Unikernel: {}", text));
                    }
                    FRAME_STATUS_VPN_RESULT => {
                        let text = String::from_utf8_lossy(&payload);
                        vpn_status_poller.set_text(&format!("🔒 VPN Return: {}", text));
                    }
                    _ => {}
                }
            }
        }
    });

    container
}

