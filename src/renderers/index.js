import Armor from "./Armor.svelte"
import MagicItem from "./MagicItem.svelte"
import Monster from "./Monster.svelte"
import PlayerClass from "./PlayerClass.svelte"
import Race from "./Race.svelte"
import Spell from "./Spell.svelte"
import Weapon from "./Weapon.svelte"

const renderers = {
  armor: Armor,
  classes: PlayerClass,
  magicitems: MagicItem,
  monsters: Monster,
  races: Race,
  spells: Spell,
  weapons: Weapon,
}

export default renderers