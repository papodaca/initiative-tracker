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

pub struct SceneImageList {
    pub container: gtk::Box,
    list: gtk::ListBox,
    store: SharedStore,
}

impl SceneImageList {
    /// Build the list UI before the real [`StateStore`] is available.
    pub fn build() -> Self {
        let store: SharedStore = Rc::new(RefCell::new(None));

        let container = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(8)
            .build();

        let list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .css_classes(["boxed-list", "scene-image-list"])
            .build();

        let add_btn = gtk::Button::builder()
            .label("Add Images")
            .tooltip_text("Add scene images (copied into app data)")
            .css_classes(["success"])
            .halign(gtk::Align::Start)
            .build();
        add_btn.update_property(&[gtk::accessible::Property::Label("Add scene images")]);

        container.append(&list);
        container.append(&add_btn);

        add_btn.connect_clicked(glib::clone!(
            #[strong]
            store,
            #[weak]
            container,
            move |_| {
                let Some(bound) = store.borrow().clone() else {
                    eprintln!("initiative-tracker: Add Images before store is ready");
                    return;
                };
                let Some(parent) = container.root().and_downcast::<gtk::Window>() else {
                    eprintln!("initiative-tracker: Add Images needs a window parent");
                    return;
                };
                open_images_dialog(&parent, bound);
            }
        ));

        Self {
            container,
            list,
            store,
        }
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

    // Fixed-size slot: loading a scaled Texture (not set_filename) so GtkPicture
    // does not adopt the full-resolution image's natural size and stretch the row.
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

    let thumb = gtk::Picture::builder()
        .content_fit(gtk::ContentFit::Contain)
        .can_shrink(true)
        .width_request(THUMB_WIDTH)
        .height_request(THUMB_HEIGHT)
        .hexpand(false)
        .vexpand(false)
        .css_classes(["scene-image-thumb-picture"])
        .build();
    if PathBuf::from(&image.path).is_file() {
        match load_thumbnail_texture(&image.path) {
            Some(texture) => thumb.set_paintable(Some(&texture)),
            None => eprintln!(
                "initiative-tracker: thumbnail decode failed for {}: {}",
                image.name, image.path
            ),
        }
    } else {
        eprintln!(
            "initiative-tracker: thumbnail missing for {}: {}",
            image.name, image.path
        );
    }
    thumb_slot.set_tooltip_text(Some(&format!("Make {} active", image.name)));

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
    thumb_slot.append(&thumb);

    let name = gtk::EditableLabel::new(&image.name);
    name.add_css_class("scene-image-name");
    name.set_hexpand(true);
    name.set_alignment(0.0);
    wire_rename(&name, store, &image.id);

    outer.append(&thumb_slot);
    outer.append(&name);
    row.set_child(Some(&outer));
    row
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

fn open_images_dialog(parent: &impl IsA<gtk::Window>, store: StateStore) {
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