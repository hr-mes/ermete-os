use gtk4::prelude::*;

/// CSS embedded per lo stile Glassmorphism elegante dello Store
pub const STORE_GLASS_CSS: &str = r#"
.window-glass {
    background-color: rgba(18, 18, 26, 0.70);
    backdrop-filter: blur(40px);
    color: #e2e8f0;
}

.store-sidebar {
    background-color: rgba(26, 27, 38, 0.50);
    border-radius: 24px;
    backdrop-filter: blur(30px);
    border: 1px solid rgba(255, 255, 255, 0.12);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
}

.central-stack {
    background-color: rgba(30, 41, 59, 0.3);
    border-radius: 24px;
    backdrop-filter: blur(20px);
    border: 1px solid rgba(255, 255, 255, 0.08);
    box-shadow: inset 0 0 10px rgba(255, 255, 255, 0.05);
}

.store-brand-title {
    font-weight: 800;
    font-size: 18px;
    color: #ffffff;
}

.nav-btn {
    background: transparent;
    border: none;
    border-radius: 10px;
    padding: 10px 14px;
    color: #cbd5e1;
    font-weight: 500;
    font-size: 14px;
    transition: all 0.2s ease-in-out;
}

.nav-btn:hover {
    background-color: rgba(255, 255, 255, 0.08);
    color: #ffffff;
}

.nav-btn:active, .nav-btn:checked {
    background-color: rgba(59, 130, 246, 0.25);
    color: #60a5fa;
    border: 1px solid rgba(96, 165, 250, 0.3);
}

.sidebar-divider {
    background-color: rgba(255, 255, 255, 0.06);
    margin: 12px 4px;
}

.sidebar-sep {
    background-color: rgba(255, 255, 255, 0.08);
    margin: 8px 0;
}

.hero-banner {
    background: linear-gradient(135deg, rgba(37, 99, 235, 0.3) 0%, rgba(147, 51, 234, 0.25) 100%);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 20px;
    padding: 28px;
    backdrop-filter: blur(16px);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
}

.hero-title {
    font-size: 24px;
    font-weight: 800;
    color: #ffffff;
}

.hero-subtitle {
    font-size: 14px;
    color: #94a3b8;
}

.hero-btn-primary {
    background-color: #2563eb;
    color: #ffffff;
    border-radius: 10px;
    padding: 8px 16px;
    font-weight: 600;
    border: none;
}

.hero-btn-primary:hover {
    background-color: #1d4ed8;
}

.hero-btn-secondary {
    background-color: rgba(255, 255, 255, 0.1);
    color: #ffffff;
    border-radius: 10px;
    padding: 8px 16px;
    font-weight: 600;
    border: 1px solid rgba(255, 255, 255, 0.15);
}

.hero-btn-secondary:hover {
    background-color: rgba(255, 255, 255, 0.18);
}

.glass-search {
    background-color: rgba(30, 41, 59, 0.6);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    color: #f8fafc;
    padding: 8px 14px;
}

.pill-btn {
    background-color: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 20px;
    color: #cbd5e1;
    padding: 6px 14px;
    font-size: 13px;
    font-weight: 500;
}

.pill-btn:hover {
    background-color: rgba(255, 255, 255, 0.12);
    color: #ffffff;
}

.section-heading {
    font-size: 18px;
    font-weight: 700;
    color: #f1f5f9;
    margin-top: 8px;
}

.store-card {
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 20px;
    padding: 18px;
    backdrop-filter: blur(24px);
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
    transition: transform 0.3s ease, background 0.3s ease, box-shadow 0.3s ease;
}

.store-card:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(96, 165, 250, 0.5);
    box-shadow: 0 12px 30px rgba(0, 0, 0, 0.3);
    transform: translateY(-4px);
}

.app-card-title {
    font-weight: 700;
    font-size: 15px;
    color: #ffffff;
}

.app-card-category {
    font-size: 12px;
    color: #94a3b8;
}

.app-card-rating {
    font-size: 12px;
    color: #fbbf24;
    font-weight: 600;
}

.app-card-summary {
    font-size: 13px;
    color: #cbd5e1;
}

.btn-install {
    background-color: #3b82f6;
    color: #ffffff;
    border-radius: 8px;
    border: none;
    padding: 6px;
    font-weight: 600;
    font-size: 13px;
}

.btn-install:hover {
    background-color: #2563eb;
}

.btn-installed {
    background-color: rgba(255, 255, 255, 0.1);
    color: #34d399;
    border-radius: 8px;
    border: 1px solid rgba(52, 211, 153, 0.3);
    padding: 6px;
    font-weight: 600;
    font-size: 13px;
}

.placeholder-box {
    background-color: rgba(30, 41, 59, 0.3);
    border-radius: 16px;
    border: 1px dashed rgba(255, 255, 255, 0.15);
    margin: 32px;
    padding: 48px;
}

.placeholder-title {
    font-size: 22px;
    font-weight: 700;
    color: #f8fafc;
}

.placeholder-sub {
    font-size: 14px;
    color: #94a3b8;
}
"#;

/// Carica le regole CSS personalizzate nel display predefinito GTK4
pub fn load_glass_css() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(STORE_GLASS_CSS);

    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
