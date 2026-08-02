use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct InitiativeTrackerWindow {}

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
            obj.set_content(Some(&toolbar));
        }
    }

    impl WidgetImpl for InitiativeTrackerWindow {}
    impl WindowImpl for InitiativeTrackerWindow {}
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
}