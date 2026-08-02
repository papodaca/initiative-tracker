use adw::prelude::*;
use gtk::glib;
use gtk::prelude::{IsA, RangeExt};

use crate::persistence::{SettingsUpdate, StateStore};
use crate::theme::{apply_theme, theme_from_index, theme_to_index};

/// Present campaign settings as an Adwaita preferences dialog.
/// Changes apply only when the user clicks Save Changes.
pub fn present_settings(parent: &impl IsA<gtk::Widget>, store: StateStore) {
    let state = store.state();
    let campaign = state.current().cloned().unwrap_or_default();

    let dialog = adw::PreferencesDialog::builder()
        .title("Campaign Settings")
        .search_enabled(false)
        .build();

    let page = adw::PreferencesPage::new();
    let group = adw::PreferencesGroup::new();

    let theme_model = gtk::StringList::new(&["System", "Light", "Dark"]);
    let theme_row = adw::ComboRow::builder()
        .title("Theme")
        .model(&theme_model)
        .selected(theme_to_index(state.theme))
        .build();

    let scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 1.0, 5.0, 0.1);
    scale.set_value(state.display_size.clamp(1.0, 5.0));
    scale.set_draw_value(true);
    scale.set_value_pos(gtk::PositionType::Right);
    scale.set_hexpand(true);
    scale.set_size_request(160, -1);
    let size_row = adw::ActionRow::builder()
        .title("Display Size")
        .subtitle("Presenter text scale")
        .build();
    size_row.set_activatable(false);
    size_row.add_suffix(&scale);

    let name_row = adw::EntryRow::builder()
        .title("Campaign Name")
        .text(state.current_campaign.as_str())
        .build();

    let show_init = adw::SwitchRow::builder()
        .title("Show Initiative Roll")
        .active(campaign.show_initiative_roll)
        .build();

    let auto_hide = adw::SwitchRow::builder()
        .title("Auto-Hide Inactive Turns")
        .subtitle("Hide dead combatants on the Presenter")
        .active(campaign.auto_hide_inactive)
        .build();

    let save_row = adw::ButtonRow::builder()
        .title("Save Changes")
        .build();
    save_row.add_css_class("suggested-action");

    group.add(&theme_row);
    group.add(&size_row);
    group.add(&name_row);
    group.add(&show_init);
    group.add(&auto_hide);
    group.add(&save_row);
    page.add(&group);
    dialog.add(&page);

    save_row.connect_activated(glib::clone!(
        #[weak]
        dialog,
        #[weak]
        theme_row,
        #[weak]
        scale,
        #[weak]
        name_row,
        #[weak]
        show_init,
        #[weak]
        auto_hide,
        move |_| {
            let update = SettingsUpdate {
                campaign_name: name_row.text().to_string(),
                theme: theme_from_index(theme_row.selected()),
                display_size: scale.value(),
                show_initiative_roll: show_init.is_active(),
                auto_hide_inactive: auto_hide.is_active(),
            };
            if let Err(e) = store.apply_settings(update.clone()) {
                eprintln!("initiative-tracker: save settings failed: {e}");
                return;
            }
            apply_theme(update.theme);
            dialog.close();
        }
    ));

    dialog.present(Some(parent));
}