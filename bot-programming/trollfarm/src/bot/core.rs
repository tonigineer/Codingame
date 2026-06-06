use crate::game::{Action, Game, Side, Tree, Troll};
use crate::utils::{CARDINALS, Position, bfs_distance_map};
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};

pub struct Bot {
    #[allow(dead_code)]
    pub side: Side,
    pub actions: Vec<Action>,
}

impl Bot {
    #[must_use]
    pub fn new() -> Self {
        Self {
            side: Side::Me,
            actions: Vec::new(),
        }
    }

    pub fn reset_turn(&mut self, game: &mut Game) {
        self.actions.clear();

        let blocked = HashSet::new();
        game.shack_dist_map =
            crate::utils::bfs_distance_map(game.shack(Side::Me), &game.grid, &blocked);
        game.opp_shack_dist_map =
            crate::utils::bfs_distance_map(game.shack(Side::Opp), &game.grid, &blocked);

        for troll in &mut game.trolls {
            troll.dist_map = crate::utils::bfs_distance_map(troll.position, &game.grid, &blocked);
        }
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
pub fn tree_occupied_by_others(tree: &Tree, troll: &Troll, game: &Game) -> bool {
    game.trolls
        .iter()
        .any(|t| t.side == Side::Me && t.id != troll.id && t.position == tree.position)
}
