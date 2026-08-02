use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use crate::window::InitiativeTrackerWindow;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct InitiativeTrackerApplication {}

    #[glib::object_subclass]
    impl ObjectSubclass for InitiativeTrackerApplication {
        const NAME: &'static str = "InitiativeTrackerApplication";
        type Type = super::InitiativeTrackerApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for InitiativeTrackerApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["<Control>q"]);
        }
    }

    impl ApplicationImpl for InitiativeTrackerApplication {
        fn activate(&self) {
            let application = self.obj();
            let window = application.active_window().unwrap_or_else(|| {
                let window = InitiativeTrackerWindow::new(&*application);
                window.upcast()
            });
            window.present();
        }
    }

    impl GtkApplicationImpl for InitiativeTrackerApplication {}
    impl AdwApplicationImpl for InitiativeTrackerApplication {}
}

glib::wrapper! {
    pub struct InitiativeTrackerApplication(ObjectSubclass<imp::InitiativeTrackerApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl InitiativeTrackerApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .property("resource-base-path", "/im/apodaca/InitiativeTracker")
            .build()
    }

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        self.add_action_entries([quit_action]);
    }
}
