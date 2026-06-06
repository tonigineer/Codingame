use crate::bot::{Bot, Candidate};
use crate::bot::params;
use crate::bot::strat_harassment::tree_occupied_by_others;
use crate::game::{Troll, Action, Game, ResourceType, Side, TrainCost, Tree, TreeType};
use crate::utils::{CARDINALS, Position, bfs_distance_map};
use std::cmp::Reverse;
use std::collections::HashSet;

const WOOD_PTS: i32 = 4;
const FRUIT_PTS: i32 = 1;
const MAX_SIZE: i32 = 4;

impl Bot {
    pub fn economy_candidates(troll: &Troll, game: &Game, out: &mut Vec<Candidate>) {
        if troll.has_cargo() {
            out.push(Bot::economy_drop(troll, game));
        }

        for tree in &game.trees {
            if tree_occupied_by_others(tree, troll, game) {
                continue;
            }
            Bot::economy_tree(troll, tree, game, out);
        }

        // Fetch a banana seed from the shack to expand (only when not already
        // carrying one — a held seed should be planted, not stockpiled).
        if troll.carry_banana == 0
            && troll.free_capacity() > 0
            && game.inventory(Side::Me).banana.amount > 0
            && let Some(c) = Bot::pick_candidate(troll, game)
        {
            out.push(c);
        }

        // Plant a carried seed on the nearest free home cell.
        if troll.carry_banana > 0
            && let Some(c) = Bot::plant_candidate(troll, game)
        {
            out.push(c);
        }
    }

    fn economy_drop(troll: &Troll, game: &Game) -> Candidate {
        let speed = troll.movement_speed.max(1);
        let to_shack = Bot::dist(game, &game.shack_dist_map, troll.position) / speed;

        let carried_pts = troll.carry_wood * WOOD_PTS
            + (troll.carry_plum + troll.carry_lemon + troll.carry_apple) * FRUIT_PTS;

        #[allow(clippy::cast_precision_loss)]
        let score = carried_pts as f32 / (to_shack + 1) as f32;

        let action = if game.is_adjacent_to_shack(troll) {
            Action::Drop(troll.id)
        } else {
            Action::Move(troll.id, game.shack(Side::Me))
        };

        Candidate {
            troll_id: troll.id,
            action,
            #[allow(clippy::cast_possible_truncation)]
            score: score as i32,
            tree: None,
        }
    }

    /// Push chop / co-chop / harvest candidates for one tree (economy scale).
    fn economy_tree(troll: &Troll, tree: &Tree, game: &Game, out: &mut Vec<Candidate>) {
        let speed = troll.movement_speed.max(1);
        let travel_raw = Bot::dist(&game, &troll.dist_map, tree.position);
        if travel_raw == i32::MAX {
            return; // unreachable
        }
        let travel = travel_raw / speed;
        let ret = Bot::dist(&game, &game.shack_dist_map, tree.position) / speed;
        let free = troll.free_capacity();
        if free == 0 {
            return; // can collect nothing; banking (economy_drop) covers this
        }

        // Is an opponent felling this tree right now?
        let opp: Vec<&Troll> = game
            .trolls
            .iter()
            .filter(|t| t.side == Side::Opp && t.position == tree.position && t.chop_power > 0)
            .collect();

        if opp.is_empty() {
            // Chop for wood — but only a tree that has reached max size AND
            // borne fruit. Wood equals size, so a growing tree is worth far more
            // felled later; and requiring a fruit first guarantees the grove a
            // reseed window (harvest competes with chop) before it is liquidated.
            if troll.chop_power > 0 && tree.size >= MAX_SIZE && tree.fruits > 0 {
                let chop = tree.health / troll.chop_power;
                let wood = tree.size.min(free);
                #[allow(clippy::cast_precision_loss)]
                let score = (wood * WOOD_PTS) as f32 / (travel + chop + ret).max(1) as f32;
                out.push(Bot::tree_action(troll, tree.position, score));
            }
            // Harvest fruit. A harvested banana is a *renewable* seed (the tree
            // survives and re-fruits): score the whole reinvest cycle (harvest →
            // walk to a free cell → plant) at `grove_value`, so it keeps seeds
            // flowing yet decays as the grove fills and yields to chopping. A
            // non-banana, or a full grove, is just banked fruit points.
            if tree.fruits > 0 && troll.harvest_power > 0 {
                let gain = tree.fruits.min(free).min(troll.harvest_power);
                let cell = (tree.typ == TreeType::Banana)
                    .then(|| Bot::nearest_free_cell(game, troll))
                    .flatten();
                #[allow(clippy::cast_precision_loss)]
                let (value, cost) = match cell {
                    Some((_, cell_d)) => {
                        (params::get().grove_value, travel + 1 + cell_d / speed + 1)
                    }
                    None => ((gain * FRUIT_PTS) as f32, travel + 1 + ret),
                };
                #[allow(clippy::cast_precision_loss)]
                let score = value / cost.max(1) as f32;
                out.push(Bot::harvest_action(troll, tree.position, score));
            }
        } else if troll.chop_power > 0 {
            // Co-chop: only choppers landing the killing turn share the wood, so
            // it is worth arriving iff we get there by the time the opp fells it.
            let opp_power: i32 = opp.iter().map(|t| t.chop_power).sum();
            let opp_kill = tree.health.div_euclid(opp_power.max(1)).max(0)
                + i32::from(tree.health % opp_power.max(1) != 0);
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
            let our_arrive = (travel_raw as u32).div_ceil(speed as u32) as i32;
            if our_arrive <= opp_kill {
                #[allow(clippy::cast_precision_loss)]
                let share = (tree.size as f32 / (opp.len() + 1) as f32).min(free as f32);
                let cost = (opp_kill.max(our_arrive) + ret + 1).max(1);
                #[allow(clippy::cast_precision_loss)]
                let score = (share * WOOD_PTS as f32) / cost as f32;
                out.push(Bot::tree_action(troll, tree.position, score));
            }
        }
    }

    /// Fetch a banana seed from the shack, scored over the full pick→plant trip
    /// (so it falls off as the nearest free cell gets farther).
    fn pick_candidate(troll: &Troll, game: &Game) -> Option<Candidate> {
        let p = params::get();
        let speed = troll.movement_speed.max(1);
        let (_, cell_dist) = Bot::nearest_free_cell(game, troll)?;
        let to_shack = Bot::dist(game, &game.shack_dist_map, troll.position) / speed;

        // pick + walk-to-shack + walk-to-cell + plant
        let cost = to_shack + 1 + cell_dist / speed + 1;
        #[allow(clippy::cast_precision_loss)]
        let score = p.grove_value / cost.max(1) as f32;

        let action = if game.is_adjacent_to_shack(troll) {
            Action::Pick(troll.id, TreeType::Banana)
        } else {
            Action::Move(troll.id, game.shack(Side::Me))
        };

        Some(Candidate {
            troll_id: troll.id,
            action,
            #[allow(clippy::cast_possible_truncation)]
            score: score as i32,
            tree: None,
        })

        // Some(Bot::candidate(troll.id, action, score, None))
    }

    /// Plant the carried seed on the nearest free home cell. The travel term is
    /// where the grove's natural size limit lives: once near cells are full the
    /// walk grows and a grown tree's chop score overtakes planting.
    fn plant_candidate(troll: &Troll, game: &Game) -> Option<Candidate> {
        let p = params::get();
        let speed = troll.movement_speed.max(1);
        let (cell, cell_dist) = Bot::nearest_free_cell(game, troll)?;

        let cost = cell_dist / speed + 1;
        #[allow(clippy::cast_precision_loss)]
        let score = p.grove_value / cost.max(1) as f32;

        let action = if troll.position == cell {
            Action::Plant(troll.id, TreeType::Banana)
        } else {
            Action::Move(troll.id, cell)
        };

        Some(Candidate {
            troll_id: troll.id,
            action,
            #[allow(clippy::cast_possible_truncation)]
            score: score as i32,
            tree: None,
        })
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
    pub fn nearest_free_cell(game: &Game, troll: &Troll) -> Option<(Position, i32)> {
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
                let n = shack + c;
                game.grid.contains(n) && matches!(game.grid[n], b'.' | b'A' | b'B' | b'P' | b'L')
            })
            .count();

        map.iter()
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
            // Nearest cell first; break ties by the cell farthest from the
            // opponent shack, then prefer one beside water (faster regrowth).
            .min_by_key(|&(pos, d)| {
                let opp = Bot::dist(game, &game.opp_shack_dist_map, pos);
                (d, Reverse(opp), i32::from(!game.is_near_water(pos)))
            })
    }

    /// A chop-targeted candidate: emit `Chop` when already on the tree, else
    /// `Move` toward it (claimed via `tree` so two trolls don't share it).
    fn tree_action(troll: &Troll, pos: Position, score: f32) -> Candidate {
        let action = if troll.position == pos {
            Action::Chop(troll.id)
        } else {
            Action::Move(troll.id, pos)
        };

        Candidate {
            troll_id: troll.id,
            action,
            #[allow(clippy::cast_possible_truncation)]
            score: score as i32,
            tree: Some(pos),
        }
    }

    /// A harvest-targeted candidate: `Harvest` when on the tree, else `Move`.
    fn harvest_action(troll: &Troll, pos: Position, score: f32) -> Candidate {
        let action = if troll.position == pos {
            Action::Harvest(troll.id)
        } else {
            Action::Move(troll.id, pos)
        };

        Candidate {
            troll_id: troll.id,
            action,
            #[allow(clippy::cast_possible_truncation)]
            score: score as i32,
            tree: Some(pos),
        }
    }
}
