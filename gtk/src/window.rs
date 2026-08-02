use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use gtk::gio;

use crate::dialogs::{present_add_campaign, present_settings};
use crate::domain::to_title_case;
use crate::persistence::StateStore;
use crate::theme::apply_theme;

mod imp {
    use super::*;
    use std::cell::{Cell, OnceCell};

    #[derive(Debug, Default)]
    pub struct InitiativeTrackerWindow {
        pub store: OnceCell<StateStore>,
        pub campaign_dropdown: OnceCell<gtk::DropDown>,
        pub status_label: OnceCell<gtk::Label>,
        pub updating_dropdown: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for InitiativeTrackerWindow {
        const NAME: &'static str = "InitiativeTrackerWindow";
        type Type = super::InitiativeTrackerWindow;
        type ParentType = adw::ApplicationWindow;
    }

    impl ObjectImpl for InitiativeTrackerWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            obj.set_title(Some("Initiative Tracker: Console"));
            obj.set_default_width(540);
            obj.set_default_height(640);

            let toolbar = adw::ToolbarView::new();
            let header = adw::HeaderBar::new();

            let dropdown = gtk::DropDown::from_strings(&[]);
            dropdown.set_hexpand(true);
            dropdown.set_halign(gtk::Align::Fill);
            dropdown.set_size_request(160, -1);
            header.set_title_widget(Some(&dropdown));
            let _ = self.campaign_dropdown.set(dropdown);

            let add_btn = gtk::Button::from_icon_name("list-add-symbolic");
            add_btn.set_tooltip_text(Some("Add campaign"));
            add_btn.add_css_class("flat");
            header.pack_start(&add_btn);

            let settings_content = adw::ButtonContent::builder()
                .icon_name("emblem-system-symbolic")
                .label("Settings")
                .build();
            let settings_btn = gtk::Button::builder()
                .child(&settings_content)
                .tooltip_text("Settings")
                .css_classes(["flat"])
                .build();
            header.pack_end(&settings_btn);

            toolbar.add_top_bar(&header);

            let scrolled = gtk::ScrolledWindow::builder()
                .hscrollbar_policy(gtk::PolicyType::Never)
                .vexpand(true)
                .build();

            let clamp = adw::Clamp::builder()
                .maximum_size(560)
                .tightening_threshold(400)
                .build();

            let body = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .spacing(12)
                .margin_top(12)
                .margin_bottom(12)
                .margin_start(12)
                .margin_end(12)
                .build();

            body.append(&placeholder_expander(
                "Presenter & Media",
                "Presenter window and scene images arrive in later phases.",
            ));
            body.append(&placeholder_expander(
                "Add Combatant",
                "PC / NPC / Monster form arrives in Phase 3.",
            ));

            let visibility = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .spacing(6)
                .homogeneous(true)
                .build();
            for label in ["Initiative", "Enemy HP", "Player HP"] {
                let btn = gtk::Button::with_label(label);
                btn.set_sensitive(false);
                btn.add_css_class("pill");
                visibility.append(&btn);
            }
            body.append(&visibility);

            let list_stub = adw::PreferencesGroup::builder()
                .title("Combatants")
                .description("Combatant list and edits arrive in Phase 3.")
                .build();
            let status = gtk::Label::builder()
                .label("Loading state…")
                .wrap(true)
                .xalign(0.0)
                .margin_top(8)
                .margin_bottom(8)
                .margin_start(12)
                .margin_end(12)
                .selectable(true)
                .build();
            // Prefer a simple status row inside the body under the group title.
            let status_frame = gtk::Frame::new(None);
            status_frame.set_child(Some(&status));
            let _ = self.status_label.set(status);
            body.append(&list_stub);
            body.append(&status_frame);

            clamp.set_child(Some(&body));
            scrolled.set_child(Some(&clamp));
            toolbar.set_content(Some(&scrolled));

            let footer = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .spacing(8)
                .margin_top(8)
                .margin_bottom(8)
                .margin_start(12)
                .margin_end(12)
                .build();
            let loop_row = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .spacing(8)
                .build();
            let prev = gtk::Button::from_icon_name("media-skip-backward-symbolic");
            prev.set_sensitive(false);
            let next = gtk::Button::with_label("Next Turn");
            next.set_sensitive(false);
            next.add_css_class("suggested-action");
            next.set_hexpand(true);
            loop_row.append(&prev);
            loop_row.append(&next);
            footer.append(&loop_row);

            let secondary = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .spacing(6)
                .homogeneous(true)
                .build();
            for label in ["Start", "End", "Long Rest", "Clear Monsters"] {
                let btn = gtk::Button::with_label(label);
                btn.set_sensitive(false);
                secondary.append(&btn);
            }
            footer.append(&secondary);

            let footer_bar = gtk::ActionBar::new();
            footer_bar.set_center_widget(Some(&footer));
            toolbar.add_bottom_bar(&footer_bar);

            obj.set_content(Some(&toolbar));

            add_btn.connect_clicked(glib::clone!(
                #[weak]
                obj,
                move |_| {
                    obj.on_add_campaign();
                }
            ));
            settings_btn.connect_clicked(glib::clone!(
                #[weak]
                obj,
                move |_| {
                    obj.on_settings();
                }
            ));

            if let Some(dropdown) = self.campaign_dropdown.get() {
                dropdown.connect_notify_local(
                    Some("selected"),
                    glib::clone!(
                        #[weak]
                        obj,
                        move |_, _| {
                            obj.on_campaign_selected();
                        }
                    ),
                );
            }

            obj.load_store();
        }
    }

    impl WidgetImpl for InitiativeTrackerWindow {}
    impl WindowImpl for InitiativeTrackerWindow {
        fn close_request(&self) -> glib::Propagation {
            if let Some(store) = self.store.get() {
                if let Err(e) = store.save() {
                    eprintln!("initiative-tracker: save on shutdown failed: {e}");
                }
            }
            self.parent_close_request()
        }
    }
    impl ApplicationWindowImpl for InitiativeTrackerWindow {}
    impl AdwApplicationWindowImpl for InitiativeTrackerWindow {}
}

fn placeholder_expander(title: &str, body: &str) -> gtk::Expander {
    let expander = gtk::Expander::builder()
        .label(title)
        .expanded(false)
        .build();
    let label = gtk::Label::builder()
        .label(body)
        .wrap(true)
        .xalign(0.0)
        .margin_top(8)
        .margin_bottom(8)
        .css_classes(["dim-label"])
        .build();
    expander.set_child(Some(&label));
    expander
}

glib::wrapper! {
    pub struct InitiativeTrackerWindow(ObjectSubclass<imp::InitiativeTrackerWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl InitiativeTrackerWindow {
    pub fn new(app: &crate::application::InitiativeTrackerApplication) -> Self {
        glib::Object::builder()
            .property("application", app)
            .build()
    }

    fn store(&self) -> Option<&StateStore> {
        self.imp().store.get()
    }

    fn on_add_campaign(&self) {
        let Some(store) = self.store().cloned() else {
            return;
        };
        present_add_campaign(self, store);
    }

    fn on_settings(&self) {
        let Some(store) = self.store().cloned() else {
            return;
        };
        present_settings(self, store);
    }

    fn on_campaign_selected(&self) {
        let imp = self.imp();
        if imp.updating_dropdown.get() {
            return;
        }
        let Some(store) = imp.store.get() else {
            return;
        };
        let Some(dropdown) = imp.campaign_dropdown.get() else {
            return;
        };
        let selected = dropdown.selected();
        let state = store.state();
        if let Some(name) = state.campaigns.get(selected as usize) {
            if name != &state.current_campaign {
                if let Err(e) = store.set_current_campaign(name) {
                    eprintln!("initiative-tracker: switch campaign failed: {e}");
                }
            }
        }
    }

    fn refresh_from_store(&self) {
        let imp = self.imp();
        let Some(store) = imp.store.get() else {
            return;
        };
        let state = store.state();
        apply_theme(state.theme);

        if let Some(dropdown) = imp.campaign_dropdown.get() {
            imp.updating_dropdown.set(true);
            let labels: Vec<String> = state
                .campaigns
                .iter()
                .map(|c| to_title_case(c))
                .collect();
            let refs: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();
            let model = gtk::StringList::new(&refs);
            dropdown.set_model(Some(&model));
            if let Some(idx) = state
                .campaigns
                .iter()
                .position(|c| c == &state.current_campaign)
            {
                dropdown.set_selected(idx as u32);
            }
            imp.updating_dropdown.set(false);
        }

        if let Some(label) = imp.status_label.get() {
            let camp = state.current();
            let n_players = camp.map(|c| c.players.len()).unwrap_or(0);
            label.set_label(&format!(
                "Campaign: {}\nCombatants: {n_players}\nTheme: {:?}\nDisplay size: {:.1}\nShow initiative roll: {}\nAuto-hide inactive: {}",
                state.current_campaign,
                state.theme,
                state.display_size,
                camp.map(|c| c.show_initiative_roll).unwrap_or(true),
                camp.map(|c| c.auto_hide_inactive).unwrap_or(false),
            ));
        }
    }

    fn load_store(&self) {
        let imp = self.imp();
        match StateStore::with_default_path() {
            Ok(store) => match store.load() {
                Ok(()) => {
                    let store_for_sub = store.clone();
                    let _ = imp.store.set(store);
                    let window = self.clone();
                    store_for_sub.subscribe(move || {
                        window.refresh_from_store();
                    });
                    self.refresh_from_store();
                }
                Err(e) => {
                    if let Some(label) = imp.status_label.get() {
                        label.set_label(&format!("Failed to load state:\n{e}"));
                    }
                }
            },
            Err(e) => {
                if let Some(label) = imp.status_label.get() {
                    label.set_label(&format!("Failed to resolve data path:\n{e}"));
                }
            }
        }
    }
}