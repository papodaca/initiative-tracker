use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use crate::domain::visible_combatants;
use crate::persistence::StateStore;

mod imp {
    use super::*;
    use std::cell::OnceCell;

    #[derive(Debug, Default)]
    pub struct InitiativeTrackerWindow {
        pub store: OnceCell<StateStore>,
        pub status_label: OnceCell<gtk::Label>,
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
            obj.set_default_height(600);

            let header = adw::HeaderBar::new();
            let toolbar = adw::ToolbarView::new();
            toolbar.add_top_bar(&header);

            let status = gtk::Label::builder()
                .label("Loading state…")
                .wrap(true)
                .margin_top(24)
                .margin_bottom(24)
                .margin_start(24)
                .margin_end(24)
                .halign(gtk::Align::Start)
                .valign(gtk::Align::Start)
                .selectable(true)
                .build();
            toolbar.set_content(Some(&status));
            let _ = self.status_label.set(status);

            obj.set_content(Some(&toolbar));
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

    fn load_store(&self) {
        let imp = self.imp();
        let label = imp
            .status_label
            .get()
            .expect("status label set in constructed");

        match StateStore::with_default_path() {
            Ok(store) => match store.load() {
                Ok(()) => {
                    let state = store.state();
                    let campaign = state.current_campaign.clone();
                    let camp = state.current();
                    let n_players = camp.map(|c| c.players.len()).unwrap_or(0);
                    let n_visible = camp
                        .map(|c| visible_combatants(c).len())
                        .unwrap_or(0);
                    let n_campaigns = state.campaigns.len();
                    let import_note = store
                        .last_import()
                        .map(|r| {
                            format!(
                                "\nImported from {:?}\nSkipped images: {}",
                                r.source,
                                r.skipped_images.len()
                            )
                        })
                        .unwrap_or_default();
                    label.set_label(&format!(
                        "State loaded\nCampaigns: {n_campaigns}\nCurrent: {campaign}\nCombatants: {n_players} (visible: {n_visible})\nPath: {:?}{import_note}",
                        store.path()
                    ));
                    let _ = imp.store.set(store);
                }
                Err(e) => {
                    label.set_label(&format!("Failed to load state:\n{e}"));
                }
            },
            Err(e) => {
                label.set_label(&format!("Failed to resolve data path:\n{e}"));
            }
        }
    }
}