import os
import re

files_to_check = [
    "src/ui/desktop_widgets.rs",
    "src/ui/gatekeeper_prompt.rs",
    "src/ui/mission_control.rs",
    "src/ui/notifications.rs",
    "src/ui/osd.rs",
    "src/ui/powermenu.rs",
    "src/ui/privacy_prompt.rs",
    "src/ui/spotlight.rs",
    "src/ui/clipboard.rs",
    "src/greeter.rs",
]

base_dir = "/var/home/ermete/GEMINI/ermete-os/forge/specs/ermete-shell-rs/ermete-shell-rs-1.0.0/"

for f in files_to_check:
    path = os.path.join(base_dir, f)
    if not os.path.exists(path):
        continue
    with open(path, "r") as f_in:
        content = f_in.read()

    # We want to remove the block that initializes CssProvider
    # It usually looks like:
    # let provider = gtk4::CssProvider::new();
    # provider.load_from_data(...);
    # if let Some(display) = ... { style_context_add_provider_for_display(...) }
    
    # Let's just remove lines that contain CssProvider::new(), load_from_data, and style_context_add_provider_for_display
    # Wait, load_from_data spans multiple lines if it's a multiline string.
    
    # Better approach: parse lines and remove the whole provider statement.
    # Actually, regex to remove from `let provider = .*CssProvider::new();` up to `STYLE_PROVIDER_PRIORITY_APPLICATION,` and its closing `);` or `}`.
    
    pattern = r'let\s+provider\s*=\s*(?:gtk4::)?CssProvider::new\(\);.*?STYLE_PROVIDER_PRIORITY_APPLICATION,?\s*\)?\s*;?(\s*\})?'
    
    new_content = re.sub(pattern, '', content, flags=re.DOTALL)
    
    # Sometimes it's without the if let display block, like in powermenu.rs:
    # gtk4::style_context_add_provider_for_display(
    #    &gtk4::gdk::Display::default().expect("Display default"),
    #    &provider,
    #    gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    # );
    
    if new_content == content:
        # Try another pattern
        pattern2 = r'let\s+provider\s*=\s*(?:gtk4::)?CssProvider::new\(\);.*?STYLE_PROVIDER_PRIORITY_APPLICATION,?\s*\)?\s*;'
        new_content = re.sub(pattern2, '', content, flags=re.DOTALL)
        
    with open(path, "w") as f_out:
        f_out.write(new_content)
