//! Console scene-image list (Presenter & Media).
//!
//! Parity with Tauri `ImageList.svelte`: multi-file add, thumbnail activate,
//! inline rename. Removal is intentionally omitted (not in Tauri UI).

use adw::prelude::*;
use gdk_pixbuf::Pixbuf;
use gtk::gdk::Texture;
use gtk::gio;
use gtk::glib;
use gtk::prelude::EditableExt;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::domain::SceneImage;
use crate::persistence::StateStore;

const IMAGE_SUFFIXES: &[&str] = &[
    "avif", "ico", "jfif", "svg", "png", "jpeg", "jpg", "webp", "bmp", "gif",
];

/// Fixed thumb slot matching Tauri ImageList (`120px × 90px`).
const THUMB_WIDTH: i32 = 120;
const THUMB_HEIGHT: i32 = 90;

type SharedStore = Rc<RefCell<Option<StateStore>>>;

/// A button that opens an `adw::Dialog` containing the scene-image list.
///
/// `bind_store` and `refresh` keep the same contract as the old inline list
/// so `window.rs` only needs to swap the appended widget.
pub struct SceneImageButton {
    pub button: gtk::Button,
    list: gtk::ListBox,
    store: SharedStore,
}

impl SceneImageButton {
    /// Build the button and its backing dialog before the [`StateStore`] is available.
    pub fn build() -> Self {
        let store: SharedStore = Rc::new(RefCell::new(None));

        let list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .css_classes(["boxed-list", "scene-image-list"])
            .build();

        let placeholder = gtk::Label::builder()
            .label("No scene images yet. Add some to show them on the Presenter.")
            .wrap(true)
            .justify(gtk::Justification::Center)
            .margin_top(24)
            .margin_bottom(24)
            .margin_start(12)
            .margin_end(12)
            .css_classes(["dim-label"])
            .build();
        list.set_placeholder(Some(&placeholder));

        let add_content = adw::ButtonContent::builder()
            .icon_name("list-add-symbolic")
            .label("Add Images")
            .build();
        let add_btn = gtk::Button::builder()
            .child(&add_content)
            .tooltip_text("Add scene images (copied into app data)")
            .css_classes(["suggested-action"])
            .build();
        add_btn.update_property(&[gtk::accessible::Property::Label("Add scene images")]);

        let hint = gtk::Label::builder()
            .label("Click a thumbnail to show it on the Presenter. Click a name to rename.")
            .wrap(true)
            .xalign(0.0)
            .css_classes(["dim-label", "caption"])
            .build();

        let dialog_content = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(8)
            .margin_top(8)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        let scrolled = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .propagate_natural_height(true)
            .vexpand(true)
            .build();
        scrolled.set_child(Some(&list));

        dialog_content.append(&hint);
        dialog_content.append(&scrolled);

        let header = adw::HeaderBar::new();
        header.pack_start(&add_btn);

        let toolbar = adw::ToolbarView::new();
        toolbar.add_top_bar(&header);
        toolbar.set_content(Some(&dialog_content));

        let dialog = adw::Dialog::builder()
            .title("Scene Images")
            .child(&toolbar)
            .content_width(480)
            .content_height(560)
            .build();

        // "Images…" button that opens the dialog; sits in the section heading row.
        let button = gtk::Button::builder()
            .label("Images…")
            .tooltip_text("Manage scene images")
            .valign(gtk::Align::Center)
            .build();
        button.update_property(&[gtk::accessible::Property::Label("Manage scene images")]);

        button.connect_clicked(glib::clone!(
            #[strong]
            dialog,
            move |btn| {
                let Some(parent) = btn.root().and_downcast::<gtk::Window>() else {
                    eprintln!("initiative-tracker: Images dialog needs a window parent");
                    return;
                };
                dialog.present(Some(&parent));
            }
        ));

        add_btn.connect_clicked(glib::clone!(
            #[strong]
            store,
            #[weak]
            dialog,
            move |_| {
                let Some(bound) = store.borrow().clone() else {
                    eprintln!("initiative-tracker: Add Images before store is ready");
                    return;
                };
                let Some(parent) = dialog.root().and_downcast::<gtk::Window>() else {
                    eprintln!("initiative-tracker: Add Images needs a window parent");
                    return;
                };
                open_images_file_dialog(&parent, bound);
            }
        ));

        Self { button, list, store }
    }

    pub fn bind_store(&self, store: StateStore) {
        *self.store.borrow_mut() = Some(store);
    }

    pub fn refresh(&self, images: &[SceneImage]) {
        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }
        let Some(store) = self.store.borrow().clone() else {
            return;
        };
        for image in images {
            self.list.append(&build_image_row(image, &store));
        }
    }
}

fn build_image_row(image: &SceneImage, store: &StateStore) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::new();
    row.set_activatable(false);
    row.add_css_class("scene-image-row");
    if image.active {
        row.add_css_class("active");
    }

    let outer = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(10)
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();

    // Fixed 120x90 slot. The texture is pre-scaled to fit that box and the
    // Picture has can_shrink=false, so its natural size is the texture size.
    // (With can_shrink=true GtkPicture reports width-for-height from the
    // aspect ratio, which made wide images push the name column around.)
    let thumb_slot = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .width_request(THUMB_WIDTH)
        .height_request(THUMB_HEIGHT)
        .hexpand(false)
        .vexpand(false)
        .halign(gtk::Align::Start)
        .valign(gtk::Align::Center)
        .css_classes(["scene-image-thumb"])
        .build();
    thumb_slot.set_overflow(gtk::Overflow::Hidden);

    let texture = PathBuf::from(&image.path)
        .is_file()
        .then(|| load_thumbnail_texture(&image.path))
        .flatten();
    match texture {
        Some(texture) => {
            let thumb = gtk::Picture::builder()
                .paintable(&texture)
                .content_fit(gtk::ContentFit::Contain)
                .can_shrink(false)
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Center)
                .hexpand(true)
                .vexpand(true)
                .css_classes(["scene-image-thumb-picture"])
                .build();
            thumb_slot.append(&thumb);
        }
        None => {
            eprintln!(
                "initiative-tracker: thumbnail unavailable for {}: {}",
                image.name, image.path
            );
            let missing = gtk::Image::builder()
                .icon_name("image-missing-symbolic")
                .pixel_size(32)
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Center)
                .hexpand(true)
                .vexpand(true)
                .css_classes(["dim-label"])
                .build();
            thumb_slot.append(&missing);
        }
    }
    thumb_slot.set_tooltip_text(Some(&format!("Show {} on the Presenter", image.name)));

    let click = gtk::GestureClick::new();
    let id = image.id.clone();
    click.connect_released(glib::clone!(
        #[strong]
        store,
        move |_, _, _, _| {
            if let Err(e) = store.set_active_image(&id) {
                eprintln!("initiative-tracker: set active image failed: {e}");
            }
        }
    ));
    thumb_slot.add_controller(click);

    let name = gtk::EditableLabel::new(&image.name);
    name.add_css_class("scene-image-name");
    name.set_hexpand(true);
    name.set_alignment(0.0);
    name.set_tooltip_text(Some(&image.name));
    ellipsize_editable_label(&name);
    wire_rename(&name, store, &image.id);

    outer.append(&thumb_slot);
    outer.append(&name);

    if image.active {
        let check = gtk::Image::builder()
            .icon_name("object-select-symbolic")
            .tooltip_text("Shown on the Presenter")
            .valign(gtk::Align::Center)
            .css_classes(["accent"])
            .build();
        outer.append(&check);
    }

    row.set_child(Some(&outer));
    row
}

/// `GtkEditableLabel` has no ellipsize property; its display label is the
/// `GtkLabel` inside the internal `GtkStack`. Without this a long file name
/// forces the whole dialog wider than the window.
fn ellipsize_editable_label(label: &gtk::EditableLabel) {
    let mut child = label.first_child();
    while let Some(widget) = child {
        if let Some(stack) = widget.downcast_ref::<gtk::Stack>() {
            let mut page = stack.first_child();
            while let Some(inner) = page {
                if let Some(text) = inner.downcast_ref::<gtk::Label>() {
                    text.set_ellipsize(gtk::pango::EllipsizeMode::End);
                }
                page = inner.next_sibling();
            }
        }
        child = widget.next_sibling();
    }
}

fn load_thumbnail_texture(path: &str) -> Option<Texture> {
    let pixbuf = Pixbuf::from_file_at_scale(path, THUMB_WIDTH, THUMB_HEIGHT, true).ok()?;
    // gdk_texture_new_for_pixbuf is deprecated in 4.20 (pixbuf loading moves to
    // glycin); still the practical way to ship a pre-scaled thumb texture.
    #[allow(deprecated)]
    Some(Texture::for_pixbuf(&pixbuf))
}

fn wire_rename(label: &gtk::EditableLabel, store: &StateStore, id: &str) {
    let id = id.to_string();
    label.connect_notify_local(
        Some("editing"),
        glib::clone!(
            #[strong]
            store,
            move |label, _| {
                if label.is_editing() {
                    return;
                }
                let text = EditableExt::text(label).to_string();
                if let Err(e) = store.rename_image(&id, text) {
                    eprintln!("initiative-tracker: rename image failed: {e}");
                }
            }
        ),
    );
}

fn open_images_file_dialog(parent: &impl IsA<gtk::Window>, store: StateStore) {
    let filter = gtk::FileFilter::new();
    filter.set_name(Some("Images"));
    for suffix in IMAGE_SUFFIXES {
        filter.add_suffix(suffix);
    }
    filter.add_mime_type("image/*");

    let filters = gio::ListStore::new::<gtk::FileFilter>();
    filters.append(&filter);

    let dialog = gtk::FileDialog::builder()
        .title("Add Images")
        .modal(true)
        .filters(&filters)
        .build();

    dialog.open_multiple(
        Some(parent),
        None::<&gio::Cancellable>,
        move |result| match result {
            Ok(model) => {
                let mut paths: Vec<PathBuf> = Vec::new();
                for i in 0..model.n_items() {
                    let Some(item) = model.item(i) else {
                        continue;
                    };
                    let Ok(file) = item.downcast::<gio::File>() else {
                        continue;
                    };
                    if let Some(path) = file.path() {
                        paths.push(path);
                    }
                }
                if paths.is_empty() {
                    return;
                }
                if let Err(e) = store.add_images(&paths) {
                    eprintln!("initiative-tracker: add images failed: {e}");
                }
            }
            Err(e) => {
                if e.matches(gtk::DialogError::Dismissed) {
                    return;
                }
                eprintln!("initiative-tracker: image dialog failed: {e}");
            }
        },
    );
}
