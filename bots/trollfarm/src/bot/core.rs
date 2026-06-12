use crate::game::{Action, Game, ResourceType, Side, Tree, Troll};
use crate::utils::{CARDINALS, Position, bfs_distance_map};
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};

pub struct Bot {
    #[allow(dead_code)]
    pub side: Side,
    pub actions: Vec<Action>,
    /// Where each of my trolls stood at the END of the previous turn, plus
    /// whether it was ordered to move — together they detect a troll whose
    /// move the referee rejected (same cell despite a `Move`). See
    /// [`Bot::reset_turn`] for how a stuck troll re-plans.
    last_positions: HashMap<i32, Position>,
    moved_last: HashSet<i32>,
    /// Latched while the conditional-3rd-troll mission is on: a bigger enemy
    /// workforce is outscaling a lead we hold, so the trolls gather the
    /// training resources (plum/lemon/iron we otherwise never bank). Strictly
    /// additive — with the latch off, behavior is unchanged. See
    /// [`Bot::update_third_troll_mission`].
    pub(crate) third_troll_mission: bool,
    /// Turn the mission latched (deadline accounting), and the permanent
    /// abort: once the deadline passes untrained, never retry this game.
    third_troll_since: i32,
    third_troll_given_up: bool,
}

impl Bot {
    #[must_use]
    pub fn new() -> Self {
        Self {
            side: Side::Me,
            actions: Vec::new(),
            last_positions: HashMap::new(),
            moved_last: HashSet::new(),
            third_troll_mission: false,
            third_troll_since: 0,
            third_troll_given_up: false,
        }
    }

    /// The stat target for the conditional 3rd troll. Deliberately CHEAP:
    /// 1/1/0/2 (3 plum / 3 lemon / 2 apple / 6 iron) — the first version
    /// targeted 2/2/0/2 and in a 20-game bench latched, gathered, and never
    /// once completed (6+6 fruit is not collectable mid-game): all tax, no
    /// troll. Speed/carry bump to 2 only when that resource is ALREADY
    /// banked; chop stays 2 (iron is the easy resource — mined 1/turn from
    /// non-depleting mines — and chopping is the new troll's whole job).
    /// `None` when some resource is neither banked nor gatherable.
    pub(crate) fn third_troll_plan(game: &Game) -> Option<(i32, i32, i32, i32)> {
        let inv = game.inventory(Side::Me);
        let gatherable = |rt: ResourceType| {
            game.trees
                .iter()
                .any(|t| t.get_resource_type() == rt && (t.fruits > 0 || t.size == 4))
        };
        // cost for a 3rd troll is 2 + stat² per resource
        let pick = |have: i32, can_gather: bool| -> Option<i32> {
            if have >= 6 {
                Some(2)
            } else if have >= 3 || can_gather {
                Some(1)
            } else {
                None
            }
        };
        let ms = pick(inv.plum.amount, gatherable(ResourceType::Plum))?;
        let cc = pick(inv.lemon.amount, gatherable(ResourceType::Lemon))?;
        // Harvest 1 ("thief stats") costs one extra apple (3 vs 2) and lets
        // the new troll BANK the enemy-orchard fruit it squats on, not just
        // block it. Skip the upgrade only when apples are that scarce.
        let hp = i32::from(inv.apple.amount >= 3 || gatherable(ResourceType::Apple));
        if inv.apple.amount < 2 && !gatherable(ResourceType::Apple) {
            return None;
        }
        let cp = if inv.iron.amount >= 6 || game.mines().next().is_some() {
            2
        } else if inv.iron.amount >= 3 {
            1
        } else {
            return None;
        };
        Some((ms, cc, hp, cp))
    }

    /// Latch / release the conditional-3rd-troll mission.
    ///
    /// Latches ON when: we field exactly 2 trolls, the opponent fields at
    /// least [`params::Params::third_troll_opp_trolls`], our banked score
    /// leads by [`params::Params::third_troll_lead`], and more than
    /// [`params::Params::third_troll_min_turns`] turns remain. Releases when
    /// the 3rd troll exists, the time budget is gone, or the lead flips
    /// negative (converting a lead we no longer hold would dig the hole
    /// deeper). Hysteresis between the +lead latch and the 0 release stops
    /// flapping.
    pub fn update_third_troll_mission(&mut self, game: &Game) {
        let p = crate::bot::params::get();
        let lead = game.inventory(Side::Me).score() - game.inventory(Side::Opp).score();
        // Deadline: a mission that hasn't produced the troll within its budget
        // is uncompletable on this map — stop bleeding the economy, for good.
        // The clock runs from the FIRST latch and keeps running through
        // unlatched stretches, so a flapping lead (latch → lead dips → unlatch
        // → relatch …) cannot stretch the budget all game (one probe bled -67
        // exactly that way).
        if self.third_troll_since > 0
            && i32::from(game.turn) - self.third_troll_since > p.third_troll_deadline
        {
            if self.third_troll_mission {
                eprintln!("[3RD] mission ABORT (deadline) turn={}", game.turn);
            }
            self.third_troll_mission = false;
            self.third_troll_given_up = true;
            return;
        }
        if self.third_troll_given_up
            || game.troll_count(Side::Me) != 2
            || game.turns_remaining() < p.third_troll_min_turns
            || (self.third_troll_mission && lead < 0)
            || Self::third_troll_plan(game).is_none()
        {
            self.third_troll_mission = false;
            return;
        }
        if !self.third_troll_mission
            && game.troll_count(Side::Opp) >= p.third_troll_opp_trolls
            && lead >= p.third_troll_lead
        {
            eprintln!(
                "[3RD] mission ON turn={} lead={} opp_trolls={}",
                game.turn,
                lead,
                game.troll_count(Side::Opp)
            );
            self.third_troll_mission = true;
            if self.third_troll_since == 0 {
                self.third_troll_since = i32::from(game.turn);
            }
        }
    }

    pub fn reset_turn(&mut self, game: &mut Game) {
        self.actions.clear();

        let blocked = HashSet::new();
        game.shack_dist_map =
            crate::utils::bfs_distance_map(game.shack(Side::Me), &game.grid, &blocked);
        game.opp_shack_dist_map =
            crate::utils::bfs_distance_map(game.shack(Side::Opp), &game.grid, &blocked);

        // STUCK detector: a troll that was ordered to move last turn but
        // stands on the same cell had its move rejected by the referee. Only
        // such a troll gets a dist map with MY other trolls' bodies as walls,
        // so blocked-corridor targets score unreachable and it re-plans
        // around (or elsewhere) instead of bonking forever — a corridor once
        // gridlocked a troll from turn 142 to 300. Walling allies for
        // EVERYONE all the time cost ~3 margin (constant plan perturbation),
        // and walling opponents cost ~40 (see `tree_occupied_by_others`), so
        // the wall applies precisely where the referee proved it real.
        let my_bodies: Vec<(i32, Position)> = game
            .trolls
            .iter()
            .filter(|t| t.side == Side::Me)
            .map(|t| (t.id, t.position))
            .collect();
        for troll in &mut game.trolls {
            let stuck = Self::is_stuck(&self.moved_last, &self.last_positions, troll);
            let blocked: HashSet<Position> = if stuck {
                my_bodies
                    .iter()
                    .filter(|(id, _)| *id != troll.id)
                    .map(|(_, p)| *p)
                    .collect()
            } else {
                HashSet::new()
            };
            troll.dist_map = crate::utils::bfs_distance_map(troll.position, &game.grid, &blocked);
        }
    }

    /// Whether this troll was ordered to move last turn yet stands on the
    /// same cell — i.e. the referee rejected its move. Associated form (not
    /// `&self`) so callers holding partial borrows of `Bot` can use it.
    pub(crate) fn is_stuck(
        moved_last: &HashSet<i32>,
        last_positions: &HashMap<i32, Position>,
        troll: &Troll,
    ) -> bool {
        troll.side == Side::Me
            && moved_last.contains(&troll.id)
            && last_positions.get(&troll.id) == Some(&troll.position)
    }

    /// Ids of my trolls currently stuck (see [`Bot::is_stuck`]); an owned set
    /// so `resolve_movement` can consult it while mutating `self.actions`.
    pub(crate) fn stuck_ids(&self, game: &Game) -> HashSet<i32> {
        game.trolls
            .iter()
            .filter(|t| Self::is_stuck(&self.moved_last, &self.last_positions, t))
            .map(|t| t.id)
            .collect()
    }

    /// Record end-of-turn troll state for next turn's stuck detector
    /// (call after actions are finalized).
    pub fn remember_turn(&mut self, game: &Game) {
        self.last_positions = game
            .trolls
            .iter()
            .filter(|t| t.side == Side::Me)
            .map(|t| (t.id, t.position))
            .collect();
        self.moved_last = self
            .actions
            .iter()
            .filter_map(|a| match a {
                Action::Move(id, _) => Some(*id),
                _ => None,
            })
            .collect();
    }

    pub fn debug(game: &Game) {
        // DEBUG: dump both shack inventories each turn (+ our per-troll carry).
        // eval.py parses these `[INV]` lines from the replay log for trajectory,
        // composition, wasted-fruit and game-length plots.
        let me = game.inventory(Side::Me);
        let opp = game.inventory(Side::Opp);
        let carried: Vec<String> = game
            .trolls(Side::Me)
            .iter()
            .map(|t| {
                format!(
                    "t{}[P{} L{} A{} B{} I{} W{}]",
                    t.id,
                    t.carry_plum,
                    t.carry_lemon,
                    t.carry_apple,
                    t.carry_banana,
                    t.carry_iron,
                    t.carry_wood
                )
            })
            .collect();
        eprintln!(
            "[INV] turn={} me_shack[P{} L{} A{} B{} I{} W{}] \
                 opp_shack[P{} L{} A{} B{} I{} W{}] {}",
            game.turn,
            me.plum.amount,
            me.lemon.amount,
            me.apple.amount,
            me.banana.amount,
            me.iron.amount,
            me.wood.amount,
            opp.plum.amount,
            opp.lemon.amount,
            opp.apple.amount,
            opp.banana.amount,
            opp.iron.amount,
            opp.wood.amount,
            carried.join(" ")
        );
    }

    // ====================================================================
    // Shared helpers
    //
    // Cross-cutting queries used by more than one strategy module. They live
    // here, beside the per-turn BFS maps built in `reset_turn`, rather than
    // inside any single strategy file.
    // ====================================================================

    /// Distance to `pos` in a BFS map.
    ///
    /// For a passable tile this is just its stored distance, or [`i32::MAX`] if
    /// the BFS never reached it. Mines (`+`) and our shack (`0`) are **not**
    /// passable, so the BFS never enters them and they are absent from `map` —
    /// yet a troll works them from an adjacent tile (mining there, dropping at
    /// the shack). For those two cell kinds this returns the distance to the
    /// closest *reachable* cardinal neighbour, i.e. how far the troll must
    /// travel to stand beside the mine/shack, rather than [`i32::MAX`]. Returns
    /// [`i32::MAX`] when `pos` is off the grid or no neighbour was reached.
    ///
    /// # Examples
    ///
    /// ```
    /// use trollfarm::bot::Bot;
    /// use trollfarm::game::Game;
    /// use trollfarm::utils::{Position, bfs_distance_map};
    /// use std::collections::HashSet;
    ///
    /// // 0 = our shack, 1 = opp shack, + = mine — all impassable, so the BFS
    /// // distance map never contains them.
    /// let input = "\
    /// 7 3
    /// 0.....1
    /// ...+...
    /// .......
    /// 0 0 0 0 0 0
    /// 0 0 0 0 0 0
    /// 0
    /// 0";
    /// let game = Game::create_mock(input);
    ///
    /// // BFS from the bottom-left passable cell (0,2).
    /// let map = bfs_distance_map(Position::new(0, 2), &game.grid, &HashSet::new());
    ///
    /// // The origin tile of the BFS map is distance 0 from itself.
    /// assert_eq!(Bot::dist(&game, &map, Position::new(0, 2)), 0);
    ///
    /// // A normal passable tile reports its own stored distance.
    /// assert_eq!(Bot::dist(&game, &map, Position::new(6, 1)), 7);
    ///
    /// // Mine '+' at (3,1): nearest reachable neighbour is (2,1) or (3,2) at 3.
    /// assert_eq!(Bot::dist(&game, &map, Position::new(3, 1)), 3);
    ///
    /// // Our shack '0' at (0,0): nearest reachable neighbour is (0,1) at 1.
    /// assert_eq!(Bot::dist(&game, &map, Position::new(0, 0)), 1);
    ///
    /// // Opponent shack '1' is out of scope (only '+' and '0' are handled).
    /// assert_eq!(Bot::dist(&game, &map, Position::new(6, 0)), i32::MAX);
    ///
    /// // Off the grid (or genuinely unreachable) stays MAX.
    /// assert_eq!(Bot::dist(&game, &map, Position::new(99, 99)), i32::MAX);
    /// ```
    #[must_use]
    pub fn dist(game: &Game, map: &HashMap<Position, (i32, Position)>, pos: Position) -> i32 {
        if let Some((d, _)) = map.get(&pos) {
            return *d;
        }

        // Impassable work tile (mine / our shack): approach it from a neighbour.
        if game.grid.contains(pos) && matches!(game.grid[pos], b'+' | b'0') {
            return CARDINALS
                .iter()
                .filter_map(|&c| map.get(&(pos + c)).map(|(d, _)| *d))
                .min()
                .unwrap_or(i32::MAX);
        }
        i32::MAX
    }

    /// Nearest empty grass cell to our shack, found by a fresh BFS over the
    /// *current* grid. The map is recomputed here (rather than reusing the
    /// per-turn cache) so trees grown or planted this turn are accounted for,
    /// and every other troll's tile is blocked so the path routes around them.
    /// Shack-*adjacent* cells are eligible — except when the shack has only a
    /// single passable neighbour, in which case that lone cell is its only
    /// access route and is kept clear so a planted tree never obstructs the
    /// shack. Ties on distance are broken by the cell *farthest* from the
    /// opponent shack, then by proximity to water (faster regrowth). Returns
    /// `(cell, dist)`.
    ///
    /// # Examples
    ///
    /// The helper plants on each returned cell (marking it `B`) and re-queries,
    /// so successive cells surface. Walking the shack from 4 down to 1 passable
    /// neighbour shows the near ring filling and, once only the lone gateway is
    /// left, the nearest spots stepping out to distance 2.
    ///
    /// ```
    /// use trollfarm::bot::Bot;
    /// use trollfarm::game::{Game, Side};
    /// use trollfarm::utils::Position;
    ///
    /// // `nearest_free_cell` builds its own shack BFS, so the mock just needs
    /// // one of our trolls to pass in (its tile is never blocked). We plant on
    /// // each returned cell so the next-nearest surfaces.
    /// let three_nearest = |input: &str| -> Vec<(Position, i32)> {
    ///     let mut game = Game::create_mock(input);
    ///     let troll = game.trolls(Side::Me)[0].clone();
    ///     let mut out = Vec::new();
    ///     for _ in 0..3 {
    ///         let Some((cell, dist)) = Bot::nearest_free_cell(&game, &troll) else { break };
    ///         out.push((cell, dist));
    ///         game.grid[cell] = b'B';
    ///     }
    ///     out
    /// };
    ///
    /// // 4 passable neighbours: shack (2,2) in open ground — the three nearest
    /// // spots are all on the ring (distance 1). Troll parked at (0,4).
    /// let case4 = "\
    /// 8 5
    /// ........
    /// ........
    /// ..0....1
    /// ........
    /// ........
    /// 0 0 0 0 0 0
    /// 0 0 0 0 0 0
    /// 0
    /// 1
    /// 100 0 0 4 1 2 1 1 0 0 0 0 0 0";
    /// let n = three_nearest(case4);
    /// assert!(n.iter().all(|&(c, d)| d == 1 && c.manhattan(Position::new(2, 2)) == 1));
    ///
    /// // 3 passable neighbours: shack (3,0) on the top edge — three ring cells.
    /// let case3 = "\
    /// 8 3
    /// ...0...1
    /// ........
    /// ........
    /// 0 0 0 0 0 0
    /// 0 0 0 0 0 0
    /// 0
    /// 1
    /// 100 0 7 2 1 2 1 1 0 0 0 0 0 0";
    /// let n = three_nearest(case3);
    /// assert!(n.iter().all(|&(c, d)| d == 1 && c.manhattan(Position::new(3, 0)) == 1));
    ///
    /// // 2 passable neighbours: shack (0,0) in a corner — two ring cells, then
    /// // the third-nearest steps out to distance 2.
    /// let case2 = "\
    /// 8 3
    /// 0......1
    /// ........
    /// ........
    /// 0 0 0 0 0 0
    /// 0 0 0 0 0 0
    /// 0
    /// 1
    /// 100 0 7 2 1 2 1 1 0 0 0 0 0 0";
    /// let n = three_nearest(case2);
    /// assert_eq!((n[0].1, n[1].1, n[2].1), (1, 1, 2));
    /// assert_eq!(n[0].0.manhattan(Position::new(0, 0)), 1);
    /// assert_eq!(n[1].0.manhattan(Position::new(0, 0)), 1);
    /// assert_eq!(n[2].0.manhattan(Position::new(0, 0)), 2);
    ///
    /// // 1 passable neighbour: shack (1,0) is walled in by water, so only (1,1)
    /// // is adjacent. It is the gateway and kept clear, so every nearby spot is
    /// // pushed out to distance 2.
    /// let case1 = "\
    /// 8 3
    /// ~0~....1
    /// ........
    /// ........
    /// 0 0 0 0 0 0
    /// 0 0 0 0 0 0
    /// 0
    /// 1
    /// 100 0 7 2 1 2 1 1 0 0 0 0 0 0";
    /// let n = three_nearest(case1);
    /// assert!(n.iter().all(|&(_, d)| d == 2));
    /// assert!(!n.iter().any(|&(c, _)| c == Position::new(1, 1)));
    /// ```
    #[must_use]
    pub fn nearest_free_cell(game: &Game, troll: &Troll) -> Option<(Position, i32)> {
        Self::nearest_free_cells(game, troll, 1).into_iter().next()
    }

    /// The `n` free home cells nearest our shack, best first.
    ///
    /// Generalises [`Bot::nearest_free_cell`] (which is simply the first of
    /// these): same fresh shack BFS over the current grid with every other troll
    /// blocked, same lone-gateway reservation, and the empty-ground cells
    /// ordered by the same key — nearest first, ties broken by the cell farthest
    /// from the opponent shack, then by water adjacency (faster regrowth).
    /// Returns up to `n` cells as `(cell, dist)`; the ordering is exercised by
    /// the [`Bot::nearest_free_cell`] doc examples.
    #[must_use]
    pub fn nearest_free_cells(game: &Game, troll: &Troll, n: usize) -> Vec<(Position, i32)> {
        let shack = game.shack(Side::Me);

        // Recompute the shack distance map here, over the *current* grid, so
        // trees grown or planted this turn are reflected (the per-turn cache
        // misses them). Every other troll's tile is blocked so the path routes
        // around them; our own troll is the one about to move, so it never
        // blocks itself.
        let blocked: HashSet<Position> = game
            .trolls
            .iter()
            .filter(|t| t.id != troll.id)
            .map(|t| t.position)
            .collect();
        let map = bfs_distance_map(shack, &game.grid, &blocked);

        // Passable cardinal neighbours of the shack (ground or a standing tree;
        // water/iron/edge/other shack are not). With just one, that cell is the
        // shack's sole gateway and must stay free of new plantings.
        let passable_neighbours = CARDINALS
            .iter()
            .filter(|&&c| {
                let neighbour = shack + c;
                game.grid.contains(neighbour)
                    && matches!(game.grid[neighbour], b'.' | b'A' | b'B' | b'P' | b'L')
            })
            .count();

        let mut cells: Vec<(Position, i32)> = map
            .iter()
            .filter_map(|(&pos, &(d, _))| {
                if game.grid[pos] != b'.' {
                    return None; // not empty ground (shack tile, tree, water, iron)
                }
                // Reserve the lone access cell when the shack is hemmed in.
                if passable_neighbours <= 1 && pos.manhattan(shack) == 1 {
                    return None;
                }
                Some((pos, d))
            })
            .collect();

        // Nearest cell first; break ties by the cell farthest from the opponent
        // shack, then prefer one beside water (faster regrowth).
        cells.sort_by_key(|&(pos, d)| {
            let opp = Bot::dist(game, &game.opp_shack_dist_map, pos);
            (d, Reverse(opp), i32::from(!game.is_near_water(pos)))
        });
        cells.truncate(n);
        cells
    }
}

impl Default for Bot {
    fn default() -> Self {
        Self::new()
    }
}

/// Whether another of my trolls is already standing on this tree.
///
/// Used to spread workers out: a tree one of my trolls already occupies is
/// excluded from another troll's candidates, so two trolls don't converge on
/// the same trunk. Shared by both the economy and harasser strategies.
///
/// OPPONENT trolls on the tree are deliberately NOT counted, in either form:
/// excluding enemy-occupied trees wholesale cedes every contested tree
/// (-42 to -62 margin), and an adjacency-only exclusion makes the troll
/// oscillate beside a long-camped tree (-60 to -80). The "failed move" spam
/// against a camper is actually optimal waiting: moves resolve
/// simultaneously, so the step succeeds the instant the camper leaves.
pub fn tree_occupied_by_others(tree: &Tree, troll: &Troll, game: &Game) -> bool {
    game.trolls
        .iter()
        .any(|t| t.side == Side::Me && t.id != troll.id && t.position == tree.position)
}
