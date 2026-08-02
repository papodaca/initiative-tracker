---
title: Phase 3 — Combatant list & combat loop
type: port
date: 2026-08-02
phase: 3
status: planned
depends_on: [2]
---

# Phase 3 — Combatant list & combat loop

## Goal

Implement full Console combat management: add/edit/delete combatants, visibility toggles, and the sticky combat action footer — parity with Tauri Console without needing the Presenter yet.

## Scope

**In**

- Add Combatant section (name, init, max HP → PC / NPC / Monster)
- Visibility toggles: Initiative, Enemy HP, Player HP
- Combatant list with inline edit, delete, active/dead styling
- Footer: Previous, Next Turn, Start, End, Long Rest, Clear Monsters
- Initiative descending sort on relevant updates

**Out**

- Presenter rendering (Phase 4)
- Scene images (Phase 5)
- Drag-and-drop reorder (unused in current Console)

## Behavior parity (from Console / PlayerList)

### Add combatant

- Defaults: name `New {Kind}` if empty; initiative `0`; max HP `10` (`DEFAULT_HEALTH`); current HP = max HP.
- `kind`: `player` | `npc` | `monster`.
- Assign UUID; derive `dead` from HP.
- Append then sort by initiative descending.

### List (console mode: editable)

- Show initiative badge, name, kind meta (`PC`/`NPC`/`Monster`), optional `• Dead`, HP `current / max`, delete button.
- Inline edit for initiative, name, health, max health (replace `InPlaceEdit` with Adwaita-friendly edit: row activate, `GtkEditableLabel`, or popover — pick one UX and use consistently).
- On HP/max HP change, re-derive `dead`.
- Active row highlighted when `player.active`.

### Visibility toggles

Campaign booleans (names in GTK schema snake_case):

- `initiative_visible`
- `health_visible` (Enemy HP button in UI — controls monster HP presentation on Presenter)
- `enemy_health_visible` (Player HP button — controls PC/NPC HP on Presenter)

Console list today always shows full HP (`healthVisible={true}` `enemyHealthVisible={true}` on Console’s `PlayerList`). **Keep that:** toggles affect Presenter (Phase 4), but still persist when flipped in Console.

### Combat loop

| Action | Behavior |
|---|---|
| Start | `current_player = 0`; set `active` on that index only |
| Next | increment with wrap to 0; update `active` flags |
| Previous | decrement with wrap to last; update `active` flags |
| End | `current_player = null`; clear all `active` |
| Long Rest | PC + NPC `health = max_health`; re-derive dead |
| Clear Monsters | filter out `kind == monster` |

### HP visibility rules (encode now for Presenter reuse)

From `PlayerList.svelte`:

- If both health flags allow, or health visible and kind is player/npc → show `health / maxHealth`
- Else if health visible, not enemyHealthVisible, kind monster → show damage taken as `-N` (or empty when 0)
- Exact flag naming is confusing in the web UI (`healthVisible` vs `enemyHealthVisible`); **match runtime behavior**, not the label etymology. Document mapping in code comments next to the Presenter binder.

## UI mapping

| Piece | Widget suggestion |
|---|---|
| Add form | `AdwPreferencesGroup` + `AdwEntryRow` / spin buttons + 3 `GtkButton`s |
| Visibility row | `GtkBox` of `GtkToggleButton`s with icons (eye / eye-slash via `icon-name`) |
| Combatant list | `GtkListBox` + custom rows or `gio::ListStore` of row objects |
| Footer | `AdwToolbarView` bottom bar / `GtkActionBar` |
| Next Turn | Emphasized suggested action button |

## Work items

1. Bind list model to `campaign.players`.
2. Implement add + clear form fields after add.
3. Implement inline edit + delete.
4. Implement visibility toggles → store.
5. Implement footer actions → domain helpers.
6. Ensure every mutation saves + notifies listeners (for Phase 4).

## Verification

- [ ] Add PC/NPC/Monster; sort order matches initiative desc
- [ ] Edit HP to 0 → Dead; heal → not dead
- [ ] Start/Next/Prev/End active highlighting correct + wraps
- [ ] Long Rest heals only PC/NPC
- [ ] Clear Monsters removes only monsters
- [ ] Restart app: list and turn index restore
- [ ] Tauri combat flow still works independently

## Exit criteria

Console can run a full combat session with persistence; ready for Presenter subscription.
