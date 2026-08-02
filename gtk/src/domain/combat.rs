use super::{Campaign, Combatant, CombatantKind};

/// Sort combatants by initiative descending (parity with `sortList` in Console.svelte).
pub fn sort_by_initiative(players: &mut [Combatant]) {
    players.sort_by(|a, b| b.initiative.cmp(&a.initiative));
}

/// Mark the combatant at `current_player` active; clear others.
pub fn update_player_active(campaign: &mut Campaign) {
    let current = campaign.current_player;
    for (i, p) in campaign.players.iter_mut().enumerate() {
        p.active = current == Some(i);
    }
}

pub fn start_initiative(campaign: &mut Campaign) {
    campaign.current_player = Some(0);
    update_player_active(campaign);
}

pub fn end_initiative(campaign: &mut Campaign) {
    campaign.current_player = None;
    update_player_active(campaign);
}

/// Advance turn, wrapping to 0 at the end of the list.
pub fn next_turn(campaign: &mut Campaign) {
    if campaign.players.is_empty() {
        campaign.current_player = None;
        update_player_active(campaign);
        return;
    }
    let next = match campaign.current_player {
        Some(i) => i + 1,
        None => 0,
    };
    campaign.current_player = Some(if next >= campaign.players.len() {
        0
    } else {
        next
    });
    update_player_active(campaign);
}

/// Previous turn, wrapping to the last index.
pub fn previous_turn(campaign: &mut Campaign) {
    if campaign.players.is_empty() {
        campaign.current_player = None;
        update_player_active(campaign);
        return;
    }
    let prev = match campaign.current_player {
        Some(i) if i > 0 => i - 1,
        Some(_) | None => campaign.players.len() - 1,
    };
    campaign.current_player = Some(prev);
    update_player_active(campaign);
}

/// Long rest: restore HP to max for players and NPCs (monsters unchanged).
pub fn long_rest(campaign: &mut Campaign) {
    for p in &mut campaign.players {
        if matches!(p.kind, CombatantKind::Player | CombatantKind::Npc) {
            p.health = p.max_health;
            p.normalize_dead();
        }
    }
}

/// Remove all combatants with kind `monster`.
pub fn clear_monsters(campaign: &mut Campaign) {
    campaign
        .players
        .retain(|p| p.kind != CombatantKind::Monster);
    // Keep current_player in range if the list shrank.
    if let Some(i) = campaign.current_player {
        if campaign.players.is_empty() {
            campaign.current_player = None;
        } else if i >= campaign.players.len() {
            campaign.current_player = Some(campaign.players.len() - 1);
        }
    }
    update_player_active(campaign);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Combatant, CombatantKind, DEFAULT_HEALTH};

    fn sample_campaign() -> Campaign {
        let mut c = Campaign::default();
        c.players = vec![
            Combatant::new("A", CombatantKind::Player, 5, DEFAULT_HEALTH),
            Combatant::new("B", CombatantKind::Npc, 10, DEFAULT_HEALTH),
            Combatant::new("C", CombatantKind::Monster, 1, DEFAULT_HEALTH),
        ];
        c
    }

    #[test]
    fn sort_descending_initiative() {
        let mut c = sample_campaign();
        sort_by_initiative(&mut c.players);
        assert_eq!(
            c.players
                .iter()
                .map(|p| p.initiative)
                .collect::<Vec<_>>(),
            vec![10, 5, 1]
        );
        assert_eq!(c.players[0].name, "B");
    }

    #[test]
    fn turn_wrap_next_and_prev() {
        let mut c = sample_campaign();
        start_initiative(&mut c);
        assert_eq!(c.current_player, Some(0));
        assert!(c.players[0].active);

        next_turn(&mut c);
        assert_eq!(c.current_player, Some(1));
        next_turn(&mut c);
        assert_eq!(c.current_player, Some(2));
        next_turn(&mut c);
        assert_eq!(c.current_player, Some(0));

        previous_turn(&mut c);
        assert_eq!(c.current_player, Some(2));
        previous_turn(&mut c);
        assert_eq!(c.current_player, Some(1));
        previous_turn(&mut c);
        assert_eq!(c.current_player, Some(0));
        previous_turn(&mut c);
        assert_eq!(c.current_player, Some(2));
    }

    #[test]
    fn end_clears_active() {
        let mut c = sample_campaign();
        start_initiative(&mut c);
        end_initiative(&mut c);
        assert!(c.current_player.is_none());
        assert!(c.players.iter().all(|p| !p.active));
    }

    #[test]
    fn long_rest_restores_pc_npc_only() {
        let mut c = sample_campaign();
        c.players[0].health = 1;
        c.players[1].health = 2;
        c.players[2].health = 3;
        long_rest(&mut c);
        assert_eq!(c.players[0].health, c.players[0].max_health);
        assert_eq!(c.players[1].health, c.players[1].max_health);
        assert_eq!(c.players[2].health, 3);
    }

    #[test]
    fn clear_monsters_removes_only_monsters() {
        let mut c = sample_campaign();
        clear_monsters(&mut c);
        assert_eq!(c.players.len(), 2);
        assert!(c.players.iter().all(|p| p.kind != CombatantKind::Monster));
    }
}
