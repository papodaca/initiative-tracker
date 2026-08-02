use adw::prelude::*;
use gtk::gio;
use gtk::glib;
use gtk::prelude::IsA;

use crate::persistence::StateStore;

/// Present an add-campaign alert dialog. On success, creates and switches to it.
pub fn present_add_campaign(parent: &impl IsA<gtk::Widget>, store: StateStore) {
    let entry = gtk::Entry::builder()
        .placeholder_text("Campaign name")
        .activates_default(true)
        .hexpand(true)
        .build();

    let dialog = adw::AlertDialog::new(Some("Add Campaign"), Some("Enter a unique campaign name."));
    dialog.set_extra_child(Some(&entry));
    dialog.add_responses(&[("cancel", "Cancel"), ("create", "Create")]);
    dialog.set_response_appearance("create", adw::ResponseAppearance::Suggested);
    dialog.set_default_response(Some("create"));
    dialog.set_close_response("cancel");
    dialog.set_response_enabled("create", false);

    entry.connect_changed(glib::clone!(
        #[weak]
        dialog,
        move |entry| {
            let ok = !entry.text().trim().is_empty();
            dialog.set_response_enabled("create", ok);
        }
    ));

    dialog.choose(
        Some(parent),
        None::<&gio::Cancellable>,
        glib::clone!(
            #[weak]
            entry,
            move |response| {
                if response != "create" {
                    return;
                }
                let name = entry.text().trim().to_string();
                if name.is_empty() {
                    return;
                }
                match store.add_campaign(&name) {
                    Ok(true) => {
                        if let Err(e) = store.set_current_campaign(&name) {
                            eprintln!("initiative-tracker: switch campaign failed: {e}");
                        }
                    }
                    Ok(false) => {
                        eprintln!(
                            "initiative-tracker: campaign name empty or duplicate: {name}"
                        );
                    }
                    Err(e) => eprintln!("initiative-tracker: add campaign failed: {e}"),
                }
            }
        ),
    );
}
