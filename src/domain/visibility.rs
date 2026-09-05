use super::{Campaign, Combatant, CombatantKind};

/// Presenter list filter: when `auto_hide_inactive`, drop dead combatants.
/// Parity with `visiblePlayers` in `Presenter.svelte`.
pub fn visible_combatants(campaign: &Campaign) -> Vec<&Combatant> {
    if campaign.auto_hide_inactive {
        campaign.players.iter().filter(|p| !p.dead).collect()
    } else {
        campaign.players.iter().collect()
    }
}

/// How HP is rendered on the Presenter for one combatant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HpDisplay {
    Hidden,
    /// `current / max`
    Full { current: i32, max: i32 },
    /// Damage taken (`max - health`); UI shows `-N` or empty when zero.
    DamageTaken { amount: i32 },
}

/// Presenter HP visibility, matching `PlayerList.svelte` after the prop swap in
/// `Presenter.svelte`:
///
/// | Campaign field (Console label) | PlayerList prop   |
/// |--------------------------------|-------------------|
/// | `enemy_health_visible` (Player HP) | `healthVisible` |
/// | `health_visible` (Enemy HP)        | `enemyHealthVisible` |
///
/// Rules (as PlayerList evaluates them):
/// - Full HP when `(player_hp && enemy_hp) || (player_hp && kind is PC/NPC)`
/// - Damage badge when `player_hp && !enemy_hp && kind is monster`
/// - Otherwise hidden
pub fn presenter_hp_display(combatant: &Combatant, campaign: &Campaign) -> HpDisplay {
    // Presenter.svelte swaps the two campaign flags into PlayerList props.
    let health_visible = campaign.enemy_health_visible;
    let enemy_health_visible = campaign.health_visible;
    hp_display(
        combatant,
        health_visible,
        enemy_health_visible,
    )
}

/// Core HP display rules used by [`presenter_hp_display`], parameterized as
/// `PlayerList` sees `healthVisible` / `enemyHealthVisible`.
pub fn hp_display(
    combatant: &Combatant,
    health_visible: bool,
    enemy_health_visible: bool,
) -> HpDisplay {
    let is_pc_or_npc = matches!(
        combatant.kind,
        CombatantKind::Player | CombatantKind::Npc
    );
    let is_monster = combatant.kind == CombatantKind::Monster;

    if (health_visible && enemy_health_visible) || (health_visible && is_pc_or_npc) {
        HpDisplay::Full {
            current: combatant.health,
            max: combatant.max_health,
        }
    } else if health_visible && !enemy_health_visible && is_monster {
        HpDisplay::DamageTaken {
            amount: combatant.max_health - combatant.health,
        }
    } else {
        HpDisplay::Hidden
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Combatant, CombatantKind, DEFAULT_HEALTH};

    fn camp_with(player_hp: bool, enemy_hp: bool) -> Campaign {
        let mut c = Campaign::default();
        // Console labels: enemy_health_visible = Player HP, health_visible = Enemy HP
        c.enemy_health_visible = player_hp;
        c.health_visible = enemy_hp;
        c
    }

    fn pc() -> Combatant {
        Combatant::new("Hero", CombatantKind::Player, 1, DEFAULT_HEALTH)
    }

    fn monster() -> Combatant {
        let mut m = Combatant::new("Goblin", CombatantKind::Monster, 1, 20);
        m.health = 15;
        m
    }

    #[test]
    fn auto_hide_filters_dead() {
        let mut c = Campaign::default();
        let mut a = pc();
        a.health = 0;
        a.normalize_dead();
        let b = Combatant::new("Alive", CombatantKind::Player, 2, DEFAULT_HEALTH);
        c.players = vec![a, b];
        c.auto_hide_inactive = false;
        assert_eq!(visible_combatants(&c).len(), 2);
        c.auto_hide_inactive = true;
        assert_eq!(visible_combatants(&c).len(), 1);
        assert_eq!(visible_combatants(&c)[0].name, "Alive");
    }

    #[test]
    fn hp_player_only_shows_pc_full_monster_damage() {
        let c = camp_with(true, false);
        assert!(matches!(
            presenter_hp_display(&pc(), &c),
            HpDisplay::Full { current: 10, max: 10 }
        ));
        assert_eq!(
            presenter_hp_display(&monster(), &c),
            HpDisplay::DamageTaken { amount: 5 }
        );
    }

    #[test]
    fn hp_both_shows_full_for_everyone() {
        let c = camp_with(true, true);
        assert!(matches!(
            presenter_hp_display(&pc(), &c),
            HpDisplay::Full { .. }
        ));
        assert_eq!(
            presenter_hp_display(&monster(), &c),
            HpDisplay::Full {
                current: 15,
                max: 20
            }
        );
    }

    #[test]
    fn hp_enemy_only_hides_all_matching_svelte_quirk() {
        // Presenter maps so only-Enemy-HP yields healthVisible=false → Hidden.
        let c = camp_with(false, true);
        assert_eq!(presenter_hp_display(&pc(), &c), HpDisplay::Hidden);
        assert_eq!(presenter_hp_display(&monster(), &c), HpDisplay::Hidden);
    }

    #[test]
    fn hp_neither_hides() {
        let c = camp_with(false, false);
        assert_eq!(presenter_hp_display(&pc(), &c), HpDisplay::Hidden);
    }
}