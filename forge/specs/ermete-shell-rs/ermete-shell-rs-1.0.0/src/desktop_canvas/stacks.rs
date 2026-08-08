use gtk4::gio;
use gtk4::prelude::*;
use gtk4::{Align, Box, Fixed, GestureClick, Label, Orientation};
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

use super::physics::{calculate_fan_out_positions, FanLayout, Spring2D};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StackCategory {
    Documents,
    Images,
    Code,
    Archives,
    Media,
    Other,
}

impl StackCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Documents => "Documents",
            Self::Images => "Images",
            Self::Code => "Code & Scripts",
            Self::Archives => "Archives",
            Self::Media => "Media",
            Self::Other => "Other Files",
        }
    }

    pub fn icon_glyph(&self) -> &'static str {
        match self {
            Self::Documents => "📄",
            Self::Images => "🖼️",
            Self::Code => "💻",
            Self::Archives => "📦",
            Self::Media => "🎵",
            Self::Other => "📁",
        }
    }

    pub fn badge_class(&self) -> &'static str {
        match self {
            Self::Documents => "cc-circle-blue",
            Self::Images => "cc-circle-indigo",
            Self::Code => "cc-circle-emerald",
            Self::Archives => "cc-circle-amber",
            Self::Media => "cc-circle-blue",
            Self::Other => "cc-circle-indigo",
        }
    }

    pub fn from_path(path: &Path) -> Self {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "pdf" | "doc" | "docx" | "txt" | "md" | "odt" | "pages" => Self::Documents,
                "png" | "jpg" | "jpeg" | "svg" | "webp" | "gif" | "bmp" => Self::Images,
                "rs" | "py" | "js" | "ts" | "c" | "cpp" | "h" | "sh" | "json" | "yaml" | "toml" | "html" | "css" => Self::Code,
                "zip" | "tar" | "gz" | "xz" | "7z" | "rar" | "bz2" => Self::Archives,
                "mp3" | "mp4" | "mkv" | "wav" | "flac" | "ogg" | "webm" => Self::Media,
                _ => Self::Other,
            }
        } else {
            Self::Other
        }
    }
}

#[derive(Debug, Clone)]
pub struct DesktopFileItem {
    pub name: String,
    pub path: PathBuf,
    pub category: StackCategory,
}

#[derive(Debug, Clone)]
pub struct DesktopStack {
    pub category: StackCategory,
    pub items: Vec<DesktopFileItem>,
}

/// Scans ~/Desktop or populates default sample files if directory is empty
pub fn load_desktop_stacks() -> Vec<DesktopStack> {
    let desktop_dir = glib::user_special_dir(glib::UserDirectory::Desktop)
        .unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("Desktop"));

    let mut items = Vec::new();
    if desktop_dir.exists() {
        if let Ok(entries) = fs::read_dir(&desktop_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "File".to_string());
                    let category = StackCategory::from_path(&path);
                    items.push(DesktopFileItem { name, path, category });
                }
            }
        }
    }

    // Fallback sample items if Desktop is empty
    if items.is_empty() {
        items.push(DesktopFileItem {
            name: "Architecture_V2.pdf".to_string(),
            path: desktop_dir.join("Architecture_V2.pdf"),
            category: StackCategory::Documents,
        });
        items.push(DesktopFileItem {
            name: "Kernel_Spec.md".to_string(),
            path: desktop_dir.join("Kernel_Spec.md"),
            category: StackCategory::Documents,
        });
        items.push(DesktopFileItem {
            name: "Wallpaper_Singularity.png".to_string(),
            path: desktop_dir.join("Wallpaper_Singularity.png"),
            category: StackCategory::Images,
        });
        items.push(DesktopFileItem {
            name: "Screen_Capture_01.webp".to_string(),
            path: desktop_dir.join("Screen_Capture_01.webp"),
            category: StackCategory::Images,
        });
        items.push(DesktopFileItem {
            name: "ebpf_sched.rs".to_string(),
            path: desktop_dir.join("ebpf_sched.rs"),
            category: StackCategory::Code,
        });
        items.push(DesktopFileItem {
            name: "main_compositor.rs".to_string(),
            path: desktop_dir.join("main_compositor.rs"),
            category: StackCategory::Code,
        });
        items.push(DesktopFileItem {
            name: "system_rootfs.tar.gz".to_string(),
            path: desktop_dir.join("system_rootfs.tar.gz"),
            category: StackCategory::Archives,
        });
    }

    use std::collections::HashMap;
    let mut map: HashMap<StackCategory, Vec<DesktopFileItem>> = HashMap::new();
    for item in items {
        map.entry(item.category.clone()).or_default().push(item);
    }

    let mut stacks = Vec::new();
    for (category, items) in map {
        stacks.push(DesktopStack { category, items });
    }

    // Sort stacks deterministically
    stacks.sort_by_key(|s| s.category.display_name());
    stacks
}

/// Structure representing a live Desktop Stack Pile widget on the canvas
pub struct DesktopStackWidget {
    pub stack: DesktopStack,
    pub pile_box: Box,
    pub origin_x: f64,
    pub origin_y: f64,
    pub is_expanded: Rc<AtomicBool>,
    pub child_widgets: Rc<RefCell<Vec<(Box, Spring2D)>>>,
}

pub fn open_desktop_file(path: &Path) {
    let uri = format!("file://{}", path.to_string_lossy());
    if let Err(e) = gio::AppInfo::launch_default_for_uri(&uri, None::<&gio::AppLaunchContext>) {
        tracing::warn!("Failed to open file via GIO uri launch ({}), trying xdg-open", e);
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
    }
}

/// Attach interactive Desktop Stacks to a GTK4 Fixed canvas
pub fn attach_desktop_stacks_to_canvas(canvas: &Fixed, start_x: f64, start_y: f64) {
    let stacks = load_desktop_stacks();
    let mut stack_offset_y = start_y;

    for stack in stacks {
        let stack_category = stack.category.clone();
        let items_count = stack.items.len();

        let pile_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(10)
            .css_classes(vec!["desktop-stack-pile"])
            .build();

        let icon_label = Label::builder()
            .label(stack_category.icon_glyph())
            .css_classes(vec![stack_category.badge_class()])
            .build();

        let title_label = Label::builder()
            .label(stack_category.display_name())
            .css_classes(vec!["cc-label-main"])
            .halign(Align::Start)
            .build();

        let count_badge = Label::builder()
            .label(&format!("{}", items_count))
            .css_classes(vec!["desktop-stack-badge"])
            .build();

        let info_vbox = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(2)
            .build();
        info_vbox.append(&title_label);

        pile_box.append(&icon_label);
        pile_box.append(&info_vbox);
        pile_box.append(&count_badge);

        let is_expanded = Rc::new(AtomicBool::new(false));
        let child_widgets: Rc<RefCell<Vec<(Box, Spring2D)>>> = Rc::new(RefCell::new(Vec::new()));

        canvas.put(&pile_box, start_x, stack_offset_y);

        let stack_origin_x = start_x;
        let stack_origin_y = stack_offset_y;

        // Build item widgets for fan-out
        let items_clone = stack.items.clone();
        let canvas_weak = canvas.downgrade();
        let child_widgets_clone = child_widgets.clone();
        let is_expanded_clone = is_expanded.clone();
        let pile_box_clone = pile_box.clone();

        let toggle_fan_out = move || {
            let Some(canvas) = canvas_weak.upgrade() else { return };
            let expanded = is_expanded_clone.load(Ordering::SeqCst);
            let next_expanded = !expanded;
            is_expanded_clone.store(next_expanded, Ordering::SeqCst);

            let mut children = child_widgets_clone.borrow_mut();

            if next_expanded {
                // If initializing fan-out widgets for first time
                if children.is_empty() {
                    let targets = calculate_fan_out_positions(
                        stack_origin_x,
                        stack_origin_y,
                        items_clone.len(),
                        180.0,
                        FanLayout::Grid,
                    );

                    for (idx, item) in items_clone.iter().enumerate() {
                        let item_card = Box::builder()
                            .orientation(Orientation::Horizontal)
                            .spacing(8)
                            .css_classes(vec!["desktop-stack-item"])
                            .build();

                        let item_icon = Label::builder()
                            .label(item.category.icon_glyph())
                            .build();

                        let item_title = Label::builder()
                            .label(&item.name)
                            .css_classes(vec!["cc-label-sub"])
                            .build();

                        item_card.append(&item_icon);
                        item_card.append(&item_title);

                        let file_path = item.path.clone();
                        let click_gesture = GestureClick::new();
                        click_gesture.connect_pressed(move |_, _, _, _| {
                            open_desktop_file(&file_path);
                        });
                        item_card.add_controller(click_gesture);

                        let (_tx, _ty) = targets[idx];
                        let spring = Spring2D::new(stack_origin_x, stack_origin_y, 180.0, 14.0);
                        canvas.put(&item_card, stack_origin_x, stack_origin_y);
                        children.push((item_card, spring));
                    }
                }

                // Set targets to fanned-out positions
                let targets = calculate_fan_out_positions(
                    stack_origin_x,
                    stack_origin_y,
                    children.len(),
                    180.0,
                    FanLayout::Grid,
                );
                for (idx, (item_widget, spring)) in children.iter_mut().enumerate() {
                    let (tx, ty) = targets[idx];
                    spring.set_target(tx, ty);
                    item_widget.set_visible(true);
                }
            } else {
                // Set targets back to origin stack position
                for (_item_widget, spring) in children.iter_mut() {
                    spring.set_target(stack_origin_x, stack_origin_y);
                }
            }

            // Frame animation tick handler
            let children_tick = child_widgets_clone.clone();
            let canvas_tick = canvas.downgrade();
            let is_expanded_tick = is_expanded_clone.clone();

            let last_frame_time = Rc::new(RefCell::new(None::<i64>));

            pile_box_clone.add_tick_callback(move |_, frame_clock| {

                let Some(canvas) = canvas_tick.upgrade() else {
                    return glib::ControlFlow::Break;
                };

                let now = frame_clock.frame_time();
                let dt = if let Some(last) = *last_frame_time.borrow() {
                    (now - last) as f64 / 1_000_000.0
                } else {
                    0.016
                };
                *last_frame_time.borrow_mut() = Some(now);

                let mut still_animating = false;
                let mut children = children_tick.borrow_mut();
                for (item_widget, spring) in children.iter_mut() {
                    let moving = spring.update(dt.min(0.05));
                    canvas.move_(item_widget, spring.x, spring.y);
                    if moving {
                        still_animating = true;
                    } else if !is_expanded_tick.load(Ordering::SeqCst) {
                        // Hide widget once collapsed back to origin
                        item_widget.set_visible(false);
                    }
                }

                if still_animating {
                    glib::ControlFlow::Continue
                } else {
                    glib::ControlFlow::Break
                }
            });
        };

        let click = GestureClick::new();
        click.connect_pressed(move |_, _, _, _| {
            toggle_fan_out();
        });
        pile_box.add_controller(click);

        stack_offset_y += 70.0;
    }
}

