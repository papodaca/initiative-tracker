mod application;
mod combat_ui;
mod dialogs;
mod domain;
mod persistence;
mod theme;
mod window;

use application::InitiativeTrackerApplication;
use gtk::{gio, glib, prelude::*};

const APP_ID: &str = "im.apodaca.InitiativeTracker";

fn main() -> glib::ExitCode {
    let app = InitiativeTrackerApplication::new(APP_ID, &gio::ApplicationFlags::empty());
    app.run()
}