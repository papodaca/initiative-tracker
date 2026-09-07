//! Console combat UI: add form, visibility toggles, combatant list, footer.
//!
//! Inline edits use [`gtk::EditableLabel`] consistently (Adwaita stand-in for
//! Svelte `InPlaceEdit`). Console always shows full HP; visibility toggles
//! persist for Presenter (Phase 4).

use adw::prelude::*;
use gtk::glib;
use gtk::prelude::EditableExt;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::domain::{kind_label, Combatant, CombatantKind, DEFAULT_HEALTH};
use crate::persistence::{CombatantPatch, StateStore};

/// Shared flag so toggle/list refresh does not re-enter store mutations.
pub type UiGuard = Rc<Cell<bool>>;

pub fn load_console_styles() {
    static LOADED: std::sync::Once = std::sync::Once::new();
    LOADED.call_once(|| {
        let provider = gtk::CssProvider::new();
        provider.load_from_string(include_str!("style.css"));
        if let Some(display) = gtk::gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    });
}

pub struct AddCombatantForm {
    pub button: gtk::Button,
}

impl AddCombatantForm {
    pub fn build(store: StateStore) -> Self {
        let content = adw::ButtonContent::builder()
            .icon_name("list-add-symbolic")
            .label("Add Combatant")
            .build();
        let button = gtk::Button::builder()
            .child(&content)
            .tooltip_text("Add a new combatant")
            .css_classes(["suggested-action"])
            .valign(gtk::Align::Center)
            .build();
        button.update_property(&[gtk::accessible::Property::Label("Add combatant")]);

        button.connect_clicked(glib::clone!(
            #[strong]
            store,
            move |btn| {
                let Some(parent) = btn.root().and_downcast::<gtk::Window>() else {
                    eprintln!("initiative-tracker: Add Combatant needs a window parent");
                    return;
                };
                present_add_combatant_dialog(&parent, store.clone());
            }
        ));

        Self { button }
    }
}

fn present_add_combatant_dialog(parent: &gtk::Window, store: StateStore) {
    let group = adw::PreferencesGroup::new();

    let name = adw::EntryRow::builder()
        .title("Name")
        .build();

    let initiative = adw::SpinRow::builder()
        .title("Initiative")
        .adjustment(
            &gtk::Adjustment::builder()
                .lower(-999.0)
                .upper(999.0)
                .step_increment(1.0)
                .page_increment(5.0)
                .value(0.0)
                .build(),
        )
        .digits(0)
        .build();

    let max_health = adw::SpinRow::builder()
        .title("Max HP")
        .adjustment(
            &gtk::Adjustment::builder()
                .lower(0.0)
                .upper(9999.0)
                .step_increment(1.0)
                .page_increment(5.0)
                .value(f64::from(DEFAULT_HEALTH))
                .build(),
        )
        .digits(0)
        .build();

    group.add(&name);
    group.add(&initiative);
    group.add(&max_health);

    // The three buttons are the submit action: each adds the combatant as
    // that kind, so they get a "Add as" heading instead of a separate OK.
    let kind_heading = gtk::Label::builder()
        .label("Add as")
        .xalign(0.0)
        .css_classes(["heading"])
        .build();

    let kind_buttons = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(6)
        .homogeneous(true)
        .build();

    let pc = gtk::Button::with_label("PC");
    pc.add_css_class("success");
    pc.set_tooltip_text(Some("Player character"));
    let npc = gtk::Button::with_label("NPC");
    npc.add_css_class("suggested-action");
    npc.set_tooltip_text(Some("Non-player character"));
    let monster = gtk::Button::with_label("Monster");
    monster.add_css_class("destructive-action");
    monster.set_tooltip_text(Some("Cleared by \"Clear Monsters\""));

    kind_buttons.append(&pc);
    kind_buttons.append(&npc);
    kind_buttons.append(&monster);

    let kind_section = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(6)
        .build();
    kind_section.append(&kind_heading);
    kind_section.append(&kind_buttons);

    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(18)
        .margin_top(12)
        .margin_bottom(18)
        .margin_start(18)
        .margin_end(18)
        .build();
    content.append(&group);
    content.append(&kind_section);

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&adw::HeaderBar::new());
    toolbar.set_content(Some(&content));

    let dialog = adw::Dialog::builder()
        .title("Add Combatant")
        .child(&toolbar)
        .content_width(400)
        .focus_widget(&name)
        .build();

    let wire = |btn: gtk::Button, kind: CombatantKind| {
        btn.connect_clicked(glib::clone!(
            #[strong]
            store,
            #[strong]
            name,
            #[strong]
            initiative,
            #[strong]
            max_health,
            #[strong]
            dialog,
            move |_| {
                let n = name.text().to_string();
                let init = initiative.value() as i32;
                let hp = max_health.value() as i32;
                if let Err(e) = store.add_combatant(n, kind, init, hp) {
                    eprintln!("initiative-tracker: add combatant failed: {e}");
                    return;
                }
                dialog.close();
            }
        ));
    };
    wire(pc, CombatantKind::Player);
    wire(npc, CombatantKind::Npc);
    wire(monster, CombatantKind::Monster);

    dialog.present(Some(parent));
}

pub struct VisibilityToggles {
    pub container: gtk::Box,
    pub initiative: gtk::ToggleButton,
    pub enemy_hp: gtk::ToggleButton,
    pub player_hp: gtk::ToggleButton,
}

impl VisibilityToggles {
    pub fn build(store: StateStore, guard: UiGuard) -> Self {
        let container = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(6)
            .homogeneous(true)
            .build();

        let initiative = make_visibility_toggle("Initiative", "view-visible-symbolic");
        let enemy_hp = make_visibility_toggle("Enemy HP", "view-visible-symbolic");
        let player_hp = make_visibility_toggle("Player HP", "view-visible-symbolic");

        container.append(&initiative);
        container.append(&enemy_hp);
        container.append(&player_hp);

        initiative.connect_toggled(glib::clone!(
            #[strong]
            store,
            #[strong]
            guard,
            move |btn| {
                if guard.get() {
                    return;
                }
                update_toggle_icon(btn);
                if let Err(e) = store.set_initiative_visible(btn.is_active()) {
                    eprintln!("initiative-tracker: initiative visibility failed: {e}");
                }
            }
        ));
        enemy_hp.connect_toggled(glib::clone!(
            #[strong]
            store,
            #[strong]
            guard,
            move |btn| {
                if guard.get() {
                    return;
                }
                update_toggle_icon(btn);
                // Campaign field `health_visible` — Console label "Enemy HP".
                if let Err(e) = store.set_health_visible(btn.is_active()) {
                    eprintln!("initiative-tracker: enemy HP visibility failed: {e}");
                }
            }
        ));
        player_hp.connect_toggled(glib::clone!(
            #[strong]
            store,
            #[strong]
            guard,
            move |btn| {
                if guard.get() {
                    return;
                }
                update_toggle_icon(btn);
                // Campaign field `enemy_health_visible` — Console label "Player HP".
                if let Err(e) = store.set_enemy_health_visible(btn.is_active()) {
                    eprintln!("initiative-tracker: player HP visibility failed: {e}");
                }
            }
        ));

        Self {
            container,
            initiative,
            enemy_hp,
            player_hp,
        }
    }

    pub fn refresh(&self, initiative_visible: bool, health_visible: bool, enemy_health_visible: bool, guard: &UiGuard) {
        guard.set(true);
        self.initiative.set_active(initiative_visible);
        self.enemy_hp.set_active(health_visible);
        self.player_hp.set_active(enemy_health_visible);
        update_toggle_icon(&self.initiative);
        update_toggle_icon(&self.enemy_hp);
        update_toggle_icon(&self.player_hp);
        guard.set(false);
    }
}

fn make_visibility_toggle(label: &str, icon: &str) -> gtk::ToggleButton {
    let content = adw::ButtonContent::builder()
        .icon_name(icon)
        .label(label)
        .build();
    let btn = gtk::ToggleButton::builder()
        .child(&content)
        .css_classes(["pill"])
        .build();
    btn
}

fn update_toggle_icon(btn: &gtk::ToggleButton) {
    let icon = if btn.is_active() {
        "view-visible-symbolic"
    } else {
        "view-conceal-symbolic"
    };
    if let Some(child) = btn.child() {
        if let Ok(content) = child.downcast::<adw::ButtonContent>() {
            content.set_icon_name(icon);
        }
    }
}

pub struct CombatantList {
    pub container: gtk::Box,
    list: gtk::ListBox,
}

impl CombatantList {
    /// `add_button` sits at the right end of the "Combatants" heading row.
    pub fn build(add_button: &gtk::Button) -> Self {
        let container = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(6)
            .build();

        let heading = gtk::Label::builder()
            .label("Combatants")
            .xalign(0.0)
            .hexpand(true)
            .css_classes(["heading"])
            .build();

        let heading_row = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(6)
            .build();
        heading_row.append(&heading);
        heading_row.append(add_button);

        let list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .css_classes(["boxed-list", "combatant-list"])
            .build();

        container.append(&heading_row);
        container.append(&list);

        Self { container, list }
    }

    pub fn refresh(&self, players: &[Combatant], store: &StateStore) {
        while let Some(child) = self.list.first_child() {
            self.list.remove(&child);
        }
        for player in players {
            self.list.append(&build_combatant_row(player, store));
        }
    }
}

fn build_combatant_row(player: &Combatant, store: &StateStore) -> gtk::ListBoxRow {
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

    let init = gtk::EditableLabel::new(&player.initiative.to_string());
    init.add_css_class("combatant-init");
    init.set_width_chars(3);
    init.set_alignment(0.5);
    init.update_property(&[gtk::accessible::Property::Label("Initiative")]);

    let info = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(2)
        .hexpand(true)
        .build();

    let name = gtk::EditableLabel::new(&player.name);
    name.add_css_class("combatant-name");
    name.set_alignment(0.0);
    name.set_hexpand(true);
    name.update_property(&[gtk::accessible::Property::Label("Combatant name")]);

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

    // Console always shows full HP (parity with Console.svelte PlayerList props).
    let hp_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(2)
        .css_classes(["combatant-hp"])
        .build();
    let health = gtk::EditableLabel::new(&player.health.to_string());
    health.set_width_chars(3);
    health.set_alignment(1.0);
    health.update_property(&[gtk::accessible::Property::Label("Current HP")]);
    let slash = gtk::Label::new(Some("/"));
    let max_health = gtk::EditableLabel::new(&player.max_health.to_string());
    max_health.set_width_chars(3);
    max_health.set_alignment(0.0);
    max_health.update_property(&[gtk::accessible::Property::Label("Max HP")]);
    hp_box.append(&health);
    hp_box.append(&slash);
    hp_box.append(&max_health);

    let delete = gtk::Button::from_icon_name("user-trash-symbolic");
    delete.set_tooltip_text(Some("Delete combatant"));
    delete.update_property(&[gtk::accessible::Property::Label("Delete combatant")]);
    delete.add_css_class("flat");
    delete.add_css_class("destructive-action");

    let id = player.id.clone();
    delete.connect_clicked(glib::clone!(
        #[strong]
        store,
        move |_| {
            if let Err(e) = store.delete_combatant(&id) {
                eprintln!("initiative-tracker: delete combatant failed: {e}");
            }
        }
    ));

    wire_editable_i32(&init, store, &player.id, |v| CombatantPatch::Initiative(v));
    wire_editable_name(&name, store, &player.id);
    wire_editable_i32(&health, store, &player.id, |v| CombatantPatch::Health(v));
    wire_editable_i32(&max_health, store, &player.id, |v| CombatantPatch::MaxHealth(v));

    outer.append(&init);
    outer.append(&info);
    outer.append(&hp_box);
    outer.append(&delete);
    row.set_child(Some(&outer));
    row
}

fn wire_editable_name(label: &gtk::EditableLabel, store: &StateStore, id: &str) {
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
                if let Err(e) = store.update_combatant(&id, CombatantPatch::Name(text)) {
                    eprintln!("initiative-tracker: rename combatant failed: {e}");
                }
            }
        ),
    );
}

fn wire_editable_i32(
    label: &gtk::EditableLabel,
    store: &StateStore,
    id: &str,
    patch: impl Fn(i32) -> CombatantPatch + 'static,
) {
    let id = id.to_string();
    let previous = Rc::new(RefCell::new(EditableExt::text(label).to_string()));
    label.connect_notify_local(
        Some("editing"),
        glib::clone!(
            #[strong]
            store,
            #[strong]
            previous,
            move |label, _| {
                if label.is_editing() {
                    *previous.borrow_mut() = EditableExt::text(label).to_string();
                    return;
                }
                let text = EditableExt::text(label).to_string();
                let Ok(value) = text.trim().parse::<i32>() else {
                    // Restore previous text on invalid input.
                    EditableExt::set_text(label, previous.borrow().as_str());
                    return;
                };
                if let Err(e) = store.update_combatant(&id, patch(value)) {
                    eprintln!("initiative-tracker: update combatant failed: {e}");
                }
            }
        ),
    );
}

pub struct CombatFooter {
    pub action_bar: gtk::ActionBar,
}

impl CombatFooter {
    pub fn build(store: StateStore) -> Self {
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
        prev.set_tooltip_text(Some("Previous turn (Ctrl+Shift+N)"));
        prev.update_property(&[gtk::accessible::Property::Label("Previous turn")]);
        let next = gtk::Button::with_label("Next Turn");
        next.set_tooltip_text(Some("Next turn (Ctrl+N)"));
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

        let start = gtk::Button::with_label("Start");
        let end = gtk::Button::with_label("End");
        let rest = gtk::Button::with_label("Long Rest");
        let clear = gtk::Button::with_label("Clear Monsters");
        clear.add_css_class("destructive-action");

        secondary.append(&start);
        secondary.append(&end);
        secondary.append(&rest);
        secondary.append(&clear);
        footer.append(&secondary);

        prev.connect_clicked(glib::clone!(
            #[strong]
            store,
            move |_| {
                if let Err(e) = store.previous_turn() {
                    eprintln!("initiative-tracker: previous turn failed: {e}");
                }
            }
        ));
        next.connect_clicked(glib::clone!(
            #[strong]
            store,
            move |_| {
                if let Err(e) = store.next_turn() {
                    eprintln!("initiative-tracker: next turn failed: {e}");
                }
            }
        ));
        start.connect_clicked(glib::clone!(
            #[strong]
            store,
            move |_| {
                if let Err(e) = store.start_initiative() {
                    eprintln!("initiative-tracker: start initiative failed: {e}");
                }
            }
        ));
        end.connect_clicked(glib::clone!(
            #[strong]
            store,
            move |_| {
                if let Err(e) = store.end_initiative() {
                    eprintln!("initiative-tracker: end initiative failed: {e}");
                }
            }
        ));
        rest.connect_clicked(glib::clone!(
            #[strong]
            store,
            move |_| {
                if let Err(e) = store.long_rest() {
                    eprintln!("initiative-tracker: long rest failed: {e}");
                }
            }
        ));
        clear.connect_clicked(glib::clone!(
            #[strong]
            store,
            move |_| {
                if let Err(e) = store.clear_monsters() {
                    eprintln!("initiative-tracker: clear monsters failed: {e}");
                }
            }
        ));

        let action_bar = gtk::ActionBar::new();
        action_bar.set_center_widget(Some(&footer));

        Self { action_bar }
    }
}