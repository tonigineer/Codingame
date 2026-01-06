use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use crate::game::{Game, MAX_GRADE_DISTANCE};
use crate::types::{Agent, Command, PlayingSide, Position, TileType};

#[derive(Debug, Clone, PartialEq)]
pub struct Bot {
    player_id: u8,
    my_agents: HashSet<Agent>,
    opp_agents: HashSet<Agent>,
    agent_commands: HashMap<Agent, Vec<(f32, Command, Command)>>,
}

impl Bot {
    pub fn new(playing_side: &PlayingSide, game: &Game) -> Self {
        let player_id = if matches!(playing_side, PlayingSide::Player) {
            game.my_id
        } else {
            game.my_id + 1 % 2
        };

        let my_agents: HashSet<Agent> = game
            .agents
            .iter()
            .filter(|(_, a)| a.player_id == game.my_id && a.alive)
            .map(|(_, agent)| (agent.clone()))
            .collect();

        let opp_agents: HashSet<Agent> = game
            .agents
            .iter()
            .filter(|(_, a)| a.player_id != game.my_id && a.alive)
            .map(|(_, agent)| (agent.clone()))
            .collect();

        let agent_commands = HashMap::with_capacity(my_agents.len());

        Bot {
            player_id,
            my_agents,
            opp_agents,
            agent_commands,
        }
    }

    pub fn play(&mut self, game: &Game) -> HashMap<Agent, (Command, Command)> {
        self.decide_on_moving(game);
        self.decide_on_grenades(game);

        self.evaluate_moves_for_points(game);
        // self.evaluate_grenades_for_damage(game);

        self.last_resort(game);

        let mut best_commands = HashMap::new();

        for agent in self.my_agents.iter() {
            if let Some(commands) = self.agent_commands.get_mut(&agent) {
                commands.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal));

                eprintln!("{}", agent.agent_id);
                for (score, cmd1, cmd2) in commands.iter() {
                    eprintln!("{} {} {}", score, cmd1, cmd2);
                }

                let (_, cmd1, cmd2) = commands.first().unwrap();
                best_commands.insert(*agent, (*cmd1, *cmd2));
            }
        }

        best_commands
    }

    fn last_resort(&mut self, game: &Game) {
        for agent in self.my_agents.iter() {
            let commands = self.agent_commands.entry(*agent).or_default();

            // Move towards enemies
            commands.push((
                1.0,
                Command::Move {
                    position: game.get_enemy_center(),
                },
                Command::HunkerDown,
            ));
        }
    }

    fn decide_on_moving(&mut self, game: &Game) {
        for agent in self.my_agents.iter() {
            let mut commands = vec![(-1.0, Command::HunkerDown, Command::None)];

            for position in &agent.position.cardinal_dirs() {
                if !game.is_valid(position) {
                    continue;
                }
                if game.get_tile(position).tile_type != TileType::Empty {
                    continue;
                }

                commands.push((
                    0.0,
                    Command::Move {
                        position: *position,
                    },
                    Command::HunkerDown,
                ));
            }

            self.agent_commands.insert(*agent, commands);
        }
    }

    fn decide_on_grenades(&mut self, game: &Game) {
        for r in 1..game.height - 1 {
            for c in 1..game.width - 1 {
                let tile = game.get_tile(&Position {
                    x: c as i16,
                    y: r as i16,
                });

                let (n_my_agents, n_opp_agents) = game.surrounding_agents(&tile.position, true);

                // enemy found and prevent friendly fire
                if n_my_agents > 0 || n_opp_agents == 0 {
                    continue;
                }

                for agent in self.my_agents.iter() {
                    eprintln!("{} {} {}", agent.agent_id, tile.position.x, tile.position.y);
                    if agent.splash_bombs == 0 {
                        continue;
                    }

                    let commands = self.agent_commands.entry(*agent).or_default();
                    let temp_commands = commands;
                    for (score, cmd1, _) in temp_commands.iter() {
                        eprintln!("{}", cmd1);
                        let agent_position = if let Some(position) = cmd1.position() {
                            position
                        } else {
                            continue;
                        };

                        let distance = agent_position.distance_to(&tile.position);
                        if distance > MAX_GRADE_DISTANCE {
                            continue;
                        }

                        let mut new_score = score + n_opp_agents as f32;
                        if tile.agent != None {
                            new_score += n_opp_agents as f32;
                        }

                        commands.push((
                            new_score,
                            *cmd1,
                            Command::Throw {
                                position: tile.position,
                            },
                        ));
                    }
                }
            }
        }
    }

    fn evaluate_moves_for_points(&mut self, game: &Game) {
        let (my_area, opp_area) = game.controlled_area();
        let ref_score = my_area - opp_area;

        for agent in self.my_agents.iter() {
            let mut temp_game = game.clone();

            if let Some(commands) = self.agent_commands.get_mut(agent) {
                for (score, cmd1, _) in commands.iter_mut() {
                    if let Some(position) = cmd1.position() {
                        temp_game
                            .agents
                            .get_mut(&agent.agent_id)
                            .unwrap()
                            .position
                            .change(position.x, position.y);

                        let (my_area, opp_area) = temp_game.controlled_area();
                        *score = ((my_area - opp_area) - ref_score) as f32;
                    }
                }

                if commands.iter().all(|(s, _, _)| *s <= 0.0) {
                    eprintln!("AAAAAA");
                }
            }
        }
    }
}
