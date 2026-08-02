use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::gio;
use gtk::glib;
use std::cell::{Cell, OnceCell, RefCell};
use std::rc::Rc;

use crate::combat_ui::{
    load_console_styles, AddCombatantForm, CombatFooter, CombatantList, UiGuard, VisibilityToggles,
};
use crate::dialogs::{present_add_campaign, present_settings};
use crate::domain::to_title_case;
use crate::persistence::StateStore;
use crate::presenter_window::PresenterWindow;
use crate::theme::apply_theme;

mod imp {
    use super::*;

    pub struct CombatWidgets {
        pub visibility: VisibilityToggles,
        pub list: CombatantList,
    }

    impl std::fmt::Debug for CombatWidgets {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("CombatWidgets").finish_non_exhaustive()
        }
    }

    pub struct PresenterControls {
        pub open_btn: gtk::Button,
        pub fullscreen_btn: gtk::Button,
        pub close_btn: gtk::Button,
    }

    impl std::fmt::Debug for PresenterControls {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("PresenterControls").finish_non_exhaustive()
        }
    }

    #[derive(Debug, Default)]
    pub struct InitiativeTrackerWindow {
        pub store: OnceCell<StateStore>,
        pub campaign_dropdown: OnceCell<gtk::DropDown>,
        pub body: OnceCell<gtk::Box>,
        pub toolbar: OnceCell<adw::ToolbarView>,
        pub combat: RefCell<Option<CombatWidgets>>,
        pub presenter_controls: OnceCell<PresenterControls>,
        pub presenter: RefCell<Option<PresenterWindow>>,
        pub presenter_visible: Cell<bool>,
        pub presenter_fullscreen: Cell<bool>,
        pub ui_guard: OnceCell<UiGuard>,
        pub updating_dropdown: Cell<bool>,
        pub combat_built: Cell<bool>,
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

            load_console_styles();

            obj.set_title(Some("Initiative Tracker: Console"));
            obj.set_default_width(540);
            obj.set_default_height(720);

            let _ = self.ui_guard.set(Rc::new(Cell::new(false)));

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

            let presenter_section = build_presenter_section(&obj);
            body.append(&presenter_section);

            clamp.set_child(Some(&body));
            scrolled.set_child(Some(&clamp));
            toolbar.set_content(Some(&scrolled));

            let _ = self.body.set(body);
            let _ = self.toolbar.set(toolbar.clone());

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
            if let Some(presenter) = self.presenter.borrow_mut().take() {
                presenter.set_hide_on_close(false);
                presenter.close();
            }
            self.parent_close_request()
        }
    }
    impl ApplicationWindowImpl for InitiativeTrackerWindow {}
    impl AdwApplicationWindowImpl for InitiativeTrackerWindow {}
}

fn build_presenter_section(window: &InitiativeTrackerWindow) -> gtk::Expander {
    let expander = gtk::Expander::builder()
        .label("Presenter & Media")
        .expanded(true)
        .build();

    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(8)
        .margin_top(8)
        .margin_bottom(8)
        .build();

    let open_btn = gtk::Button::builder()
        .label("Open Presenter Window")
        .css_classes(["suggested-action"])
        .build();

    let row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .homogeneous(true)
        .build();

    let fullscreen_btn = gtk::Button::with_label("Fullscreen");
    fullscreen_btn.add_css_class("pill");
    fullscreen_btn.set_sensitive(false);

    let close_btn = gtk::Button::with_label("Close");
    close_btn.add_css_class("destructive-action");
    close_btn.set_sensitive(false);

    row.append(&fullscreen_btn);
    row.append(&close_btn);

    let stub = gtk::Label::builder()
        .label("Scene images arrive in Phase 5. Drag the Presenter to the player display, then Fullscreen.")
        .wrap(true)
        .xalign(0.0)
        .css_classes(["dim-label"])
        .build();

    content.append(&open_btn);
    content.append(&row);
    content.append(&stub);
    expander.set_child(Some(&content));

    open_btn.connect_clicked(glib::clone!(
        #[weak]
        window,
        move |_| {
            window.open_presenter();
        }
    ));
    fullscreen_btn.connect_clicked(glib::clone!(
        #[weak]
        window,
        move |_| {
            window.toggle_presenter_fullscreen();
        }
    ));
    close_btn.connect_clicked(glib::clone!(
        #[weak]
        window,
        move |_| {
            window.close_presenter();
        }
    ));

    let _ = window.imp().presenter_controls.set(imp::PresenterControls {
        open_btn,
        fullscreen_btn,
        close_btn,
    });
    window.sync_presenter_controls();

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

    fn ensure_presenter(&self) -> Option<PresenterWindow> {
        let imp = self.imp();
        if let Some(existing) = imp.presenter.borrow().as_ref() {
            return Some(existing.clone());
        }

        let store = imp.store.get()?.clone();
        let app = self.application()?;
        let presenter = PresenterWindow::new(&app, store);

        let console = self.clone();
        presenter.set_visibility_callback(move |visible| {
            console.imp().presenter_visible.set(visible);
            if !visible {
                console.imp().presenter_fullscreen.set(false);
            }
            console.sync_presenter_controls();
        });

        let console = self.clone();
        presenter.set_fullscreen_callback(move |fullscreen| {
            console.imp().presenter_fullscreen.set(fullscreen);
            console.sync_presenter_controls();
        });

        *imp.presenter.borrow_mut() = Some(presenter.clone());
        Some(presenter)
    }

    fn open_presenter(&self) {
        let Some(presenter) = self.ensure_presenter() else {
            eprintln!("initiative-tracker: cannot open Presenter (store/app missing)");
            return;
        };
        presenter.open_presenter();
        self.imp().presenter_visible.set(true);
        self.sync_presenter_controls();
    }

    fn close_presenter(&self) {
        if let Some(presenter) = self.imp().presenter.borrow().as_ref() {
            presenter.close_presenter();
        }
        self.imp().presenter_visible.set(false);
        self.imp().presenter_fullscreen.set(false);
        self.sync_presenter_controls();
    }

    fn toggle_presenter_fullscreen(&self) {
        let Some(presenter) = self.ensure_presenter() else {
            return;
        };
        if !self.imp().presenter_visible.get() {
            presenter.open_presenter();
            self.imp().presenter_visible.set(true);
        }
        let next = !self.imp().presenter_fullscreen.get();
        presenter.set_presenter_fullscreen(next);
        self.imp().presenter_fullscreen.set(next);
        self.sync_presenter_controls();
    }

    fn sync_presenter_controls(&self) {
        let Some(controls) = self.imp().presenter_controls.get() else {
            return;
        };
        let visible = self.imp().presenter_visible.get();
        let fullscreen = self.imp().presenter_fullscreen.get();

        controls.open_btn.set_sensitive(!visible);
        controls.close_btn.set_sensitive(visible);
        controls.fullscreen_btn.set_sensitive(visible);
        controls.fullscreen_btn.set_label(if fullscreen {
            "Exit Fullscreen"
        } else {
            "Fullscreen"
        });
    }

    fn ensure_combat_ui(&self) {
        let imp = self.imp();
        if imp.combat_built.get() {
            return;
        }
        let Some(store) = imp.store.get().cloned() else {
            return;
        };
        let Some(body) = imp.body.get() else {
            return;
        };
        let Some(toolbar) = imp.toolbar.get() else {
            return;
        };
        let Some(guard) = imp.ui_guard.get().cloned() else {
            return;
        };

        let add = AddCombatantForm::build(store.clone());
        let visibility = VisibilityToggles::build(store.clone(), guard);
        let list = CombatantList::build();
        let footer = CombatFooter::build(store);

        body.append(&add.expander);
        body.append(&visibility.container);
        body.append(&list.container);
        toolbar.add_bottom_bar(&footer.action_bar);

        *imp.combat.borrow_mut() = Some(imp::CombatWidgets { visibility, list });
        imp.combat_built.set(true);
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

        self.ensure_combat_ui();

        if let Some(combat) = imp.combat.borrow().as_ref() {
            let camp = state.current().cloned().unwrap_or_default();
            if let Some(guard) = imp.ui_guard.get() {
                combat.visibility.refresh(
                    camp.initiative_visible,
                    camp.health_visible,
                    camp.enemy_health_visible,
                    guard,
                );
            }
            combat.list.refresh(&camp.players, store);
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
                    eprintln!("initiative-tracker: failed to load state: {e}");
                    if let Some(body) = imp.body.get() {
                        body.append(&error_label(&format!("Failed to load state:\n{e}")));
                    }
                }
            },
            Err(e) => {
                eprintln!("initiative-tracker: failed to resolve data path: {e}");
                if let Some(body) = imp.body.get() {
                    body.append(&error_label(&format!("Failed to resolve data path:\n{e}")));
                }
            }
        }
    }
}

fn error_label(text: &str) -> gtk::Label {
    gtk::Label::builder()
        .label(text)
        .wrap(true)
        .xalign(0.0)
        .selectable(true)
        .css_classes(["error"])
        .build()
}