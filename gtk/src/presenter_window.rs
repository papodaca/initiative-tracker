//! Player-facing Presenter window with live `StateStore` sync.
//!
//! Parity with Tauri `Presenter.svelte`: hide-on-close, F11/Esc fullscreen,
//! read-only initiative list with Presenter filters and HP rules. Scene
//! backgrounds arrive in Phase 5.

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::gdk;
use gtk::gio;
use gtk::glib;
use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;

use crate::domain::{
    kind_label, presenter_hp_display, visible_combatants, AppState, Campaign, Combatant, HpDisplay,
};
use crate::persistence::StateStore;
use crate::theme::apply_theme;

type BoolCallback = Rc<dyn Fn(bool)>;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct PresenterWindow {
        pub store: OnceCell<StateStore>,
        pub list: OnceCell<gtk::ListBox>,
        pub scale_box: OnceCell<gtk::Box>,
        pub scale_provider: OnceCell<gtk::CssProvider>,
        pub on_fullscreen: RefCell<Option<BoolCallback>>,
        pub on_visibility: RefCell<Option<BoolCallback>>,
        pub subscribed: Cell<bool>,
        pub syncing_fullscreen: Cell<bool>,
    }

    impl std::fmt::Debug for PresenterWindow {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("PresenterWindow")
                .field("subscribed", &self.subscribed)
                .field("syncing_fullscreen", &self.syncing_fullscreen)
                .finish_non_exhaustive()
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PresenterWindow {
        const NAME: &'static str = "InitiativeTrackerPresenterWindow";
        type Type = super::PresenterWindow;
        type ParentType = adw::ApplicationWindow;
    }

    impl ObjectImpl for PresenterWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            obj.set_title(Some("Initiative Tracker: Presenter"));
            obj.set_default_width(800);
            obj.set_default_height(600);
            obj.set_hide_on_close(true);

            let provider = gtk::CssProvider::new();
            if let Some(display) = gdk::Display::default() {
                gtk::style_context_add_provider_for_display(
                    &display,
                    &provider,
                    gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
                );
            }
            let _ = self.scale_provider.set(provider);

            let scale_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .valign(gtk::Align::Start)
                .halign(gtk::Align::Fill)
                .hexpand(true)
                .vexpand(true)
                .css_classes(["presenter-scale"])
                .build();

            let clamp = adw::Clamp::builder()
                .maximum_size(720)
                .tightening_threshold(480)
                .hexpand(true)
                .build();

            let list = gtk::ListBox::builder()
                .selection_mode(gtk::SelectionMode::None)
                .css_classes(["boxed-list", "combatant-list", "presenter-list"])
                .build();

            clamp.set_child(Some(&list));
            scale_box.append(&clamp);

            let scrolled = gtk::ScrolledWindow::builder()
                .hscrollbar_policy(gtk::PolicyType::Never)
                .vexpand(true)
                .child(&scale_box)
                .build();

            obj.set_content(Some(&scrolled));

            let _ = self.list.set(list);
            let _ = self.scale_box.set(scale_box);

            let key = gtk::EventControllerKey::new();
            key.connect_key_pressed(glib::clone!(
                #[weak]
                obj,
                #[upgrade_or]
                glib::Propagation::Proceed,
                move |_, keyval, _, _| {
                    if keyval == gdk::Key::F11 {
                        obj.toggle_fullscreen();
                        glib::Propagation::Stop
                    } else if keyval == gdk::Key::Escape && obj.is_fullscreen() {
                        obj.set_presenter_fullscreen(false);
                        glib::Propagation::Stop
                    } else {
                        glib::Propagation::Proceed
                    }
                }
            ));
            obj.add_controller(key);

            obj.connect_notify_local(
                Some("fullscreened"),
                glib::clone!(
                    #[weak]
                    obj,
                    move |_, _| {
                        obj.emit_fullscreen_callback();
                    }
                ),
            );

            obj.connect_show(glib::clone!(
                #[weak]
                obj,
                move |_| {
                    obj.emit_visibility_callback(true);
                }
            ));
            obj.connect_hide(glib::clone!(
                #[weak]
                obj,
                move |_| {
                    obj.emit_visibility_callback(false);
                }
            ));
        }
    }

    impl WidgetImpl for PresenterWindow {}
    impl WindowImpl for PresenterWindow {}
    impl ApplicationWindowImpl for PresenterWindow {}
    impl AdwApplicationWindowImpl for PresenterWindow {}
}

glib::wrapper! {
    pub struct PresenterWindow(ObjectSubclass<imp::PresenterWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl PresenterWindow {
    pub fn new(app: &impl IsA<gtk::Application>, store: StateStore) -> Self {
        let window: Self = glib::Object::builder()
            .property("application", app)
            .build();

        let imp = window.imp();
        let _ = imp.store.set(store.clone());

        if !imp.subscribed.get() {
            let win = window.clone();
            store.subscribe(move || {
                win.refresh_from_store();
            });
            imp.subscribed.set(true);
        }

        window.refresh_from_store();
        window
    }

    pub fn set_fullscreen_callback(&self, cb: impl Fn(bool) + 'static) {
        *self.imp().on_fullscreen.borrow_mut() = Some(Rc::new(cb));
    }

    pub fn set_visibility_callback(&self, cb: impl Fn(bool) + 'static) {
        *self.imp().on_visibility.borrow_mut() = Some(Rc::new(cb));
    }

    pub fn open_presenter(&self) {
        self.present();
    }

    pub fn close_presenter(&self) {
        self.set_presenter_fullscreen(false);
        self.set_visible(false);
    }

    pub fn toggle_fullscreen(&self) {
        self.set_presenter_fullscreen(!self.is_fullscreen());
    }

    pub fn set_presenter_fullscreen(&self, fullscreen: bool) {
        let imp = self.imp();
        if self.is_fullscreen() == fullscreen {
            return;
        }
        imp.syncing_fullscreen.set(true);
        if fullscreen {
            self.fullscreen();
        } else {
            self.unfullscreen();
        }
        imp.syncing_fullscreen.set(false);
        if let Some(cb) = imp.on_fullscreen.borrow().as_ref() {
            cb(self.is_fullscreen());
        }
    }

    fn emit_fullscreen_callback(&self) {
        if self.imp().syncing_fullscreen.get() {
            return;
        }
        if let Some(cb) = self.imp().on_fullscreen.borrow().as_ref() {
            cb(self.is_fullscreen());
        }
    }

    fn emit_visibility_callback(&self, visible: bool) {
        if let Some(cb) = self.imp().on_visibility.borrow().as_ref() {
            cb(visible);
        }
    }

    fn refresh_from_store(&self) {
        let imp = self.imp();
        let Some(store) = imp.store.get() else {
            return;
        };
        let state = store.state();
        apply_theme(state.theme);
        self.apply_display_size(state.display_size);
        self.rebind_list(&state);
    }

    fn apply_display_size(&self, size: f64) {
        let imp = self.imp();
        let Some(provider) = imp.scale_provider.get() else {
            return;
        };
        let clamped = size.clamp(1.0, 5.0);
        provider.load_from_string(&format!(
            ".presenter-scale {{ font-size: {clamped}em; }}"
        ));
    }

    fn rebind_list(&self, state: &AppState) {
        let imp = self.imp();
        let Some(list) = imp.list.get() else {
            return;
        };
        while let Some(child) = list.first_child() {
            list.remove(&child);
        }

        let Some(campaign) = state.current() else {
            list.set_visible(false);
            return;
        };

        if !campaign.initiative_visible {
            list.set_visible(false);
            return;
        }
        list.set_visible(true);

        for combatant in visible_combatants(campaign) {
            list.append(&build_presenter_row(combatant, campaign));
        }
    }
}

fn build_presenter_row(player: &Combatant, campaign: &Campaign) -> gtk::ListBoxRow {
    let row = gtk::ListBoxRow::new();
    row.add_css_class("combatant-row");
    if player.active {
        row.add_css_class("active");
    }
    if player.dead {
        row.add_css_class("dead");
    }
    row.set_activatable(false);

    let outer = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(10)
        .margin_top(4)
        .margin_bottom(4)
        .margin_start(4)
        .margin_end(4)
        .build();

    if campaign.show_initiative_roll {
        let init = gtk::Label::builder()
            .label(player.initiative.to_string())
            .width_chars(3)
            .xalign(0.5)
            .css_classes(["combatant-init"])
            .build();
        if player.active {
            init.add_css_class("presenter-init-active");
        }
        outer.append(&init);
    }

    let info = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(2)
        .hexpand(true)
        .build();

    let name = gtk::Label::builder()
        .label(&player.name)
        .xalign(0.0)
        .hexpand(true)
        .css_classes(["combatant-name"])
        .build();

    let meta_text = if player.dead {
        format!("{} • Dead", kind_label(player.kind))
    } else {
        kind_label(player.kind).to_string()
    };
    let meta = gtk::Label::builder()
        .label(&meta_text)
        .xalign(0.0)
        .css_classes(["dim-label", "combatant-meta"])
        .build();

    info.append(&name);
    info.append(&meta);
    outer.append(&info);

    match presenter_hp_display(player, campaign) {
        HpDisplay::Hidden => {}
        HpDisplay::Full { current, max } => {
            let hp = gtk::Label::builder()
                .label(format!("{current} / {max}"))
                .xalign(1.0)
                .css_classes(["combatant-hp"])
                .build();
            outer.append(&hp);
        }
        HpDisplay::DamageTaken { amount } => {
            let text = if amount == 0 {
                String::new()
            } else {
                format!("-{amount}")
            };
            let hp = gtk::Label::builder()
                .label(text)
                .xalign(1.0)
                .width_chars(4)
                .css_classes(["combatant-hp"])
                .build();
            outer.append(&hp);
        }
    }

    row.set_child(Some(&outer));
    row
}