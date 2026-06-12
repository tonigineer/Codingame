#[cfg(test)]
mod tests {
    use trollfarm::game::{Game, Side};
    use trollfarm::game::{Tree, TreeType};
    use trollfarm::utils::Position;

    const MOCK_INPUT: &str = "\
18 9
#...~.........~~~~
....~1.....#..~~~~
..#.~......#...~~~
+.~~~.............
...~..........~...
.............~~~.+
~~~...#......~.#..
~~~~..#.....0~....
~~~~.........~...#
10 10 6 9 5 0
10 10 6 9 5 0
12
PLUM 10 1 3 10 0 1
PLUM 7 7 3 10 0 1
LEMON 17 4 2 8 0 3
LEMON 0 4 2 8 0 3
LEMON 9 6 2 8 0 1
LEMON 8 2 2 8 0 1
APPLE 5 2 4 20 1 2
APPLE 12 6 4 20 1 2
BANANA 7 3 3 5 0 1
BANANA 10 5 3 5 0 1
BANANA 13 1 4 6 0 4
BANANA 4 7 4 6 0 4
2
0 1 5 1 1 1 1 1 0 0 0 0 0 0
1 0 12 7 1 1 1 1 0 0 0 0 0 0";

    #[test]
    fn test_create_mock() {
        let game = Game::create_mock(MOCK_INPUT);

        // Grid dimensions
        assert_eq!(game.width, 18);
        assert_eq!(game.height, 9);
        assert_eq!(game.turn, 1);

        // Shack positions: '0' at (12,7), '1' at (5,1)
        assert_eq!(game.shacks[0], Position::new(12, 7)); // me
        assert_eq!(game.shacks[1], Position::new(5, 1)); // opp

        // Mines: '+' at (0,3) and (17,5)
        assert_eq!(game.mines.len(), 2);
        assert!(game.mines.contains(&Position::new(0, 3)));
        assert!(game.mines.contains(&Position::new(17, 5)));

        // Inventories
        let my_inv = game.inventory(Side::Me);
        assert_eq!(my_inv.plum.amount, 10);
        assert_eq!(my_inv.lemon.amount, 10);
        assert_eq!(my_inv.apple.amount, 6);
        assert_eq!(my_inv.banana.amount, 9);
        assert_eq!(my_inv.iron.amount, 5);
        assert_eq!(my_inv.wood.amount, 0);

        let opp_inv = game.inventory(Side::Opp);
        assert_eq!(opp_inv.plum.amount, 10);
        assert_eq!(opp_inv.lemon.amount, 10);
        assert_eq!(opp_inv.apple.amount, 6);
        assert_eq!(opp_inv.banana.amount, 9);
        assert_eq!(opp_inv.iron.amount, 5);
        assert_eq!(opp_inv.wood.amount, 0);

        // Trees
        assert_eq!(game.trees.len(), 12);

        let plums: Vec<&Tree> = game
            .trees
            .iter()
            .filter(|t| t.typ == TreeType::Plum)
            .collect();
        assert_eq!(plums.len(), 2);
        assert_eq!(plums[0].position, Position::new(10, 1));
        assert_eq!(plums[0].size, 3);
        assert_eq!(plums[0].health, 10);
        assert_eq!(plums[0].fruits, 0);
        assert_eq!(plums[0].cooldown, 1);

        let apples: Vec<&Tree> = game
            .trees
            .iter()
            .filter(|t| t.typ == TreeType::Apple)
            .collect();
        assert_eq!(apples.len(), 2);
        assert_eq!(apples[0].size, 4);
        assert_eq!(apples[0].health, 20);
        assert_eq!(apples[0].fruits, 1);

        let bananas: Vec<&Tree> = game
            .trees
            .iter()
            .filter(|t| t.typ == TreeType::Banana)
            .collect();
        assert_eq!(bananas.len(), 4);

        let lemons: Vec<&Tree> = game
            .trees
            .iter()
            .filter(|t| t.typ == TreeType::Lemon)
            .collect();
        assert_eq!(lemons.len(), 4);

        // Trolls
        assert_eq!(game.trolls.len(), 2);

        let opp_troll = &game.trolls[0];
        assert_eq!(opp_troll.id, 0);
        assert_eq!(opp_troll.side, Side::Opp);
        assert_eq!(opp_troll.position, Position::new(5, 1));
        assert_eq!(opp_troll.movement_speed, 1);
        assert_eq!(opp_troll.carry_capacity, 1);
        assert_eq!(opp_troll.harvest_power, 1);
        assert_eq!(opp_troll.chop_power, 1);
        assert_eq!(opp_troll.total_carried(), 0);

        let my_troll = &game.trolls[1];
        assert_eq!(my_troll.id, 1);
        assert_eq!(my_troll.side, Side::Me);
        assert_eq!(my_troll.position, Position::new(12, 7));
        assert_eq!(my_troll.movement_speed, 1);
        assert_eq!(my_troll.carry_capacity, 1);
        assert_eq!(my_troll.harvest_power, 1);
        assert_eq!(my_troll.chop_power, 1);
        assert_eq!(my_troll.total_carried(), 0);

        // Grid: tree bytes stamped
        assert_eq!(game.grid[Position::new(10, 1)], b'P');
        assert_eq!(game.grid[Position::new(5, 2)], b'A');
        assert_eq!(game.grid[Position::new(7, 3)], b'B');
        assert_eq!(game.grid[Position::new(9, 6)], b'L');

        // Grid: terrain preserved
        assert_eq!(game.grid[Position::new(0, 0)], b'#');
        assert_eq!(game.grid[Position::new(4, 0)], b'~');
        assert_eq!(game.grid[Position::new(0, 3)], b'+');
        assert_eq!(game.grid[Position::new(1, 0)], b'.');
    }
}
