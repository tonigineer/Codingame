package com.codingame.game;

import java.util.ArrayList;
import java.util.Collections;
import java.util.Comparator;
import java.util.List;
import java.util.Random;
import java.util.Set;
import java.util.TreeSet;
import java.util.function.Function;
import java.util.stream.Collectors;
import java.util.stream.IntStream;

import com.codingame.game.Game.Move;
import com.codingame.game.action.Action;
import com.codingame.game.action.ActionType;
import com.codingame.game.grid.Coord;
import com.codingame.game.grid.GridMaker;
import com.codingame.game.grid.Tile;
import com.codingame.game.pathfinding.PathFinder;
import com.codingame.game.pathfinding.PathFinder.PathFinderResult;
import com.codingame.gameengine.core.MultiplayerGameManager;
import com.google.inject.Inject;
import com.google.inject.Singleton;

@Singleton
public class TutorialManager {
    @Inject private Game game;
    @Inject private MultiplayerGameManager<Player> gameManager;
    @Inject private PathFinder pathfinder;

    int leagueLevel;
    Random random;
    List<Player> players;

    List<Set<Agent>> balloonKillOrder;

    public boolean initTutorial(Random random, int leagueLevel) {
        this.leagueLevel = leagueLevel;
        this.balloonKillOrder = null;
        this.random = random;
        this.players = gameManager.getPlayers();
        if (leagueLevel == -1) {
            // Debugging and screenshot taking
            Game.CONTROL_ZONES = false;
            Game.ONE_PLAYER_MODE = false;

            game.grid = GridMaker.initEmpty(6, 4);
            pathfinder.setGrid(game.grid);
            int agentId = 1;
            Agent agent1 = new Agent(agentId++, AgentClass.SNIPER);
            agent1.balloons = 0;
            Player player = players.get(0);
            player.agents.add(agent1);
            agent1.owner = player;
            agent1.setPosition(new Coord(0, 1));
            game.grid.get(1, 0).setType(Tile.TYPE_LOW_COVER);
            game.grid.get(0, 3).setType(Tile.TYPE_HIGH_COVER);
            game.grid.get(4, 2).setType(Tile.TYPE_HIGH_COVER);

            agent1 = new Agent(agentId++, AgentClass.SNIPER);
            agent1.balloons = 0;
            player = players.get(1);
            player.agents.add(agent1);
            agent1.owner = player;
            agent1.setPosition(new Coord(1, 2));

            return true;
        }
        if (leagueLevel == 1) {
            Game.ONE_PLAYER_MODE = true;
            Game.CONTROL_ZONES = false;
            Game.ALLOW_SHOOT = false;
            Game.ALLOW_HUNKER = false;
            Game.ALLOW_THROW = false;
            gameManager.setMaxTurns(20);

            game.grid = GridMaker.initEmpty(8, 5);
            pathfinder.setGrid(game.grid);

            int agentId = 1;
            Agent agent1 = new Agent(agentId++, AgentClass.SNIPER);
            Agent agent2 = new Agent(agentId++, AgentClass.SNIPER);
            agent1.balloons = 0;
            agent2.balloons = 0;
            agent1.maxCooldown = 0;
            agent2.maxCooldown = 0;

            Player player = players.get(0);
            player.agents.add(agent1);
            player.agents.add(agent2);
            agent1.owner = player;
            agent2.owner = player;

            agent1.setPosition(new Coord(0, 0));
            agent2.setPosition(new Coord(0, 4));
            return true;
        } else if (leagueLevel == 2) {
            Game.CONTROL_ZONES = false;
            Game.ALLOW_HUNKER = false;
            Game.ALLOW_THROW = false;
            Game.ALLOW_MOVE = false;

            gameManager.setMaxTurns(20);

            game.grid = GridMaker.initEmpty(8, 5);
            pathfinder.setGrid(game.grid);

            int agentId = 1;

            Player player = players.get(0);
            Player boss = players.get(1);

            // Player
            for (int i = 0; i < 2; i++) {
                Agent a = new Agent(agentId++, AgentClass.SNIPER);
                player.agents.add(a);
                a.owner = player;
                a.soakingPower = 50;
                a.balloons = 0;
                a.maxCooldown = 0;
            }

            // Boss
            for (int i = 0; i < 4; i++) {
                Agent a = new Agent(agentId++, AgentClass.SNIPER);
                boss.agents.add(a);
                a.owner = boss;
                a.soakingPower = 100;
                a.balloons = 0;
                a.maxCooldown = 0;
            }

            Coord[] playerSpawn = new Coord[] {
                new Coord(3, 2),
                new Coord(4, 2)
            };

            Coord[] foeSpawn = new Coord[] {
                new Coord(2, 1),
                new Coord(2, 3),
                new Coord(5, 1),
                new Coord(5, 3)
            };

            for (int i = 0; i < player.agents.size(); i++) {
                player.agents.get(i).setPosition(playerSpawn[i]);

            }

            // shuffle foe ids to stop hardcoding
            Collections.shuffle(boss.agents, random);
            for (int i = 0; i < boss.agents.size(); i++) {
                boss.agents.get(i).setPosition(foeSpawn[i]);
                boss.agents.get(i).setWetness(90 - i * 10);
            }
            return true;
        } else if (leagueLevel == 3) {
            // Run and gun objective
            Game.CONTROL_ZONES = false;
            Game.ALLOW_THROW = false;
            Game.ALLOW_HUNKER = false;
            int w = 13;

            gameManager.setMaxTurns(1);

            game.grid = GridMaker.initEmpty(w, 5);
            pathfinder.setGrid(game.grid);

            int agentId = 1;

            Player player = players.get(0);
            Player boss = players.get(1);

            Agent a1 = new Agent(agentId++, AgentClass.SNIPER);
            player.agents.add(a1);
            a1.owner = player;
            a1.soakingPower = 50;
            a1.balloons = 0;
            a1.optimalRange = 6;
            a1.maxCooldown = 0;
            a1.setPosition(new Coord(0, 2));

            Agent a2 = new Agent(agentId++, AgentClass.SNIPER);
            player.agents.add(a2);
            a2.owner = player;
            a2.soakingPower = 50;
            a2.balloons = 0;
            a2.optimalRange = 6;
            a2.maxCooldown = 0;
            a2.setPosition(new Coord(w - 1, 2));

            // Boss
            for (int i = 0; i < 4; i++) {
                Agent b = new Agent(agentId++, AgentClass.SNIPER);
                boss.agents.add(b);
                b.owner = boss;
                b.soakingPower = 100;
                b.balloons = 0;
                b.maxCooldown = 0;
                switch (i) {
                case 0 -> b.setPosition(new Coord(4, 1));
                case 1 -> b.setPosition(new Coord(4, 3));
                case 2 -> b.setPosition(new Coord(w - 5, 1));
                case 3 -> b.setPosition(new Coord(w - 5, 3));
                }
            }

            boolean t = random.nextBoolean();
            game.grid.get(1, 1).setType(t ? Tile.TYPE_LOW_COVER : Tile.TYPE_HIGH_COVER);
            game.grid.get(1, 3).setType(!t ? Tile.TYPE_LOW_COVER : Tile.TYPE_HIGH_COVER);

            t = random.nextBoolean();
            game.grid.get(w - 2, 1).setType(t ? Tile.TYPE_LOW_COVER : Tile.TYPE_HIGH_COVER);
            game.grid.get(w - 2, 3).setType(!t ? Tile.TYPE_LOW_COVER : Tile.TYPE_HIGH_COVER);

            int foeDefense = random.nextBoolean() ? Tile.TYPE_LOW_COVER : Tile.TYPE_FLOOR;

            t = random.nextBoolean();
            game.grid.get(3, 1).setType(t ? foeDefense : Tile.TYPE_HIGH_COVER);
            game.grid.get(3, 3).setType(!t ? foeDefense : Tile.TYPE_HIGH_COVER);

            t = random.nextBoolean();
            game.grid.get(w - 4, 1).setType(t ? foeDefense : Tile.TYPE_HIGH_COVER);
            game.grid.get(w - 4, 3).setType(!t ? foeDefense : Tile.TYPE_HIGH_COVER);

            if (random.nextBoolean()) {
                game.grid.get(w / 2, 1).setType(Tile.TYPE_HIGH_COVER);
                game.grid.get(w / 2, 3).setType(Tile.TYPE_HIGH_COVER);
            }

            if (random.nextBoolean()) {
                t = random.nextBoolean();
                game.grid.get(4, 2).setType(t ? Tile.TYPE_LOW_COVER : Tile.TYPE_HIGH_COVER);
                game.grid.get(w - 5, 2).setType(!t ? Tile.TYPE_LOW_COVER : Tile.TYPE_HIGH_COVER);
            }
            if (random.nextBoolean()) {
                t = random.nextBoolean();
                game.grid.get(4, 0).setType(t ? Tile.TYPE_LOW_COVER : Tile.TYPE_HIGH_COVER);
                game.grid.get(w - 5, 0).setType(!t ? Tile.TYPE_LOW_COVER : Tile.TYPE_HIGH_COVER);
            }
            if (random.nextBoolean()) {
                t = random.nextBoolean();
                game.grid.get(4, 4).setType(t ? Tile.TYPE_LOW_COVER : Tile.TYPE_HIGH_COVER);
                game.grid.get(w - 5, 4).setType(!t ? Tile.TYPE_LOW_COVER : Tile.TYPE_HIGH_COVER);
            }
            return true;
        } else if (leagueLevel == 4) {
            Game.CONTROL_ZONES = false;
            Game.ALLOW_HUNKER = false;
            Game.ALLOW_THROW = true;
            Game.ALLOW_MOVE = true;

            gameManager.setMaxTurns(40);

            game.grid = GridMaker.initEmpty(15 + (random.nextBoolean() ? 0 : 2), 12);
            pathfinder.setGrid(game.grid);

            balloonKillOrder = new ArrayList<>(3);

            int agentId = 1;

            Player player = players.get(0);
            Player boss = players.get(1);

            for (int i = 0; i < 2; i++) {
                Agent a = new Agent(agentId++, AgentClass.SNIPER);
                player.agents.add(a);
                a.owner = player;
                a.setWetness(70);
                a.soakingPower = 10;
                a.optimalRange = 1;
                a.maxCooldown = 0;
                a.balloons = i == 0 ? 3 : 1;
                a.setPosition(new Coord(game.grid.width / 2, game.grid.height / 2 - i));
            }

            // shuffle ids
            if (random.nextBoolean()) {
                player.agents.get(0).id = 2;
                player.agents.get(1).id = 1;
            }

            List<Integer> counts = new ArrayList<>(List.of(random.nextInt(3) + 5, random.nextInt(3) + 4, 7, random.nextInt(4) + 7));
            Collections.shuffle(counts, random);
            int trappedIdx = IntStream.range(0, counts.size())
                .reduce((i, j) -> counts.get(i) < counts.get(j) ? i : j)
                .getAsInt();

            int bunkerStepX = game.grid.width - 5;
            int bunkerStepY = game.grid.height - 5;
            for (int i = 0; i < 4; ++i) {
                int count = counts.get(i);
                int bunkerX = 2 + bunkerStepX * (i / 2);
                int bunkerY = 2 + bunkerStepY * (i % 2);
                boolean trappedPlace = false;
                Set<Agent> hitlist = new TreeSet<>();

                List<Integer> xCoords = new ArrayList<>(List.of(-2, -1, 0, 1, 2));
                List<Integer> yCoords = new ArrayList<>(List.of(-2, -1, 0, 1, 2));
                Collections.shuffle(xCoords, random);
                Collections.shuffle(yCoords, random);

                for (int x : xCoords) {
                    for (int y : yCoords) {
                        Coord c = new Coord(bunkerX + x, bunkerY + y);
                        if (Math.abs(x) == 2 || Math.abs(y) == 2) {
                            game.grid.get(c).setType(Tile.TYPE_HIGH_COVER);
                        } else if (count > 0) {
                            if (trappedIdx == i && !trappedPlace) {
                                player.agents.get(1).setPosition(c);
                                trappedPlace = true;
                            } else if (count > 0) {
                                Agent b = new Agent(agentId++, AgentClass.SNIPER);
                                boss.agents.add(b);
                                b.owner = boss;
                                b.optimalRange = 64;
                                b.soakingPower = 200;
                                b.maxCooldown = 0;
                                b.setWetness(70);
                                b.setPosition(c);
                                count--;
                                if (trappedIdx != i) {
                                    hitlist.add(b);
                                }
                            }
                        }
                    }

                }
                if (!hitlist.isEmpty()) {
                    balloonKillOrder.add(hitlist);
                }
            }

            balloonKillOrder.sort(Comparator.comparingInt((Set<?> s) -> s.size()).reversed());

            return true;
        }
        return false;
    }

    public boolean league3ObjectiveComplete() {
        Coord[] runAndGunCoords = getRunAndGunObjectiveCoords();
        Agent agent1 = players.get(0).agents.get(0);
        Agent agent2 = players.get(0).agents.get(1);

        if (!agent1.hasMoveAction() || !agent2.hasMoveAction()) {
            return false;
        }

        //XXX: will return false even if the auto-pathfinder moves agent to the right place
        Coord moveTo1 = getPathFinderResult(agent1);
        Coord moveTo2 = getPathFinderResult(agent2);
        Coord at1 = agent1.getPosition();
        Coord at2 = agent2.getPosition();
        
        if (!runAndGunCoords[0].equals(moveTo1) || !runAndGunCoords[1].equals(moveTo2)) {
            if (!runAndGunCoords[0].equals(at1) || !runAndGunCoords[1].equals(at2)) {
                return false;
            }
        }

        if (!agent1.hasShootAction() || !agent2.hasShootAction()) {
            return false;
        }

        Agent target1 = game.getAgentById(agent1.getCombatAction().getAgentId());
        Agent target2 = game.getAgentById(agent2.getCombatAction().getAgentId());
        if (target1 == null || target2 == null) {
            return false;
        }

        if (
            !target1.getPosition().equals(runAndGunCoords[2])
                || !target2.getPosition().equals(runAndGunCoords[3])
        ) {
            return false;
        }

        return true;
    }

    private Coord getPathFinderResult(Agent agent) {
        PathFinderResult res = pathfinder
            .from(agent.getPosition())
            .to(agent.getMoveAction().getCoord())
            .findPath();
        if (!res.hasNextCoord()) {
            return null;
        }
        return res.getNextCoord();
    }

    // For league 3
    public Coord[] getRunAndGunObjectiveCoords() {
        if (leagueLevel == 3) {
            Coord runA = game.grid.get(1, 1).getType() == Tile.TYPE_HIGH_COVER ? new Coord(0, 1) : new Coord(0, 3);
            Coord runB = game.grid.get(game.grid.width - 2, 1).getType() == Tile.TYPE_HIGH_COVER ? new Coord(game.grid.width - 1, 1)
                : new Coord(game.grid.width - 1, 3);

            Coord gunA = game.grid.get(3, 1).getType() == Tile.TYPE_HIGH_COVER ? new Coord(4, 3) : new Coord(4, 1);
            Coord gunB = game.grid.get(game.grid.width - 4, 1).getType() == Tile.TYPE_HIGH_COVER ? new Coord(game.grid.width - 5, 3)
                : new Coord(game.grid.width - 5, 1);

            return new Coord[] {
                runA,
                runB,
                gunA,
                gunB
            };
        }
        return new Coord[0];
    }

    public void handleEnd(String[] scoreTexts) {
        if (leagueLevel == 1) {
            boolean win = league1ObjectiveComplete();
            players.get(0).setScore(win ? 0 : -1);
            players.get(1).setScore(win ? -1 : 0);
            scoreTexts[0] = win ? "objective complete" : "objective failed";
            scoreTexts[1] = "-";
        } else if (leagueLevel == 2) {
            players.get(0).setScore(players.get(0).agents.size());
            players.get(1).setScore(players.get(1).agents.size());
            scoreTexts[0] = players.get(0).getLiveAgentCount() > 0 ? "objective complete" : "objective failed";
            scoreTexts[1] = "-";
        } else if (leagueLevel == 3) {
            boolean win = league3ObjectiveComplete();
            players.get(0).setScore(win ? 0 : -1);
            players.get(1).setScore(win ? -1 : 0);
            scoreTexts[0] = win ? "objective complete" : "objective failed";
            scoreTexts[1] = "-";
        } else if (leagueLevel == 4) {
            boolean win = league4ObjectiveComplete();
            players.get(0).setScore(win ? 0 : -1);
            players.get(1).setScore(win ? -1 : 0);
            scoreTexts[0] = win ? "objective complete" : "objective failed";
            scoreTexts[1] = "-";
        }

    }

    public void changeTutorialBossActions(int turn) {
        if (leagueLevel == 2) {
            // Make sure the user has only shot the wettest enemy
            Agent wettest = players.get(1).agents.stream()
                .filter(a -> !a.dying)
                .max((a1, a2) -> Double.compare(a1.getWetness(), a2.getWetness()))
                .orElse(null);
            boolean everyAgentShotWettest = true;
            for (Agent a : players.get(0).agents) {
                if (!a.hasShootAction() || a.getCombatAction().getAgentId() != wettest.getId()) {
                    everyAgentShotWettest = false;
                }
            }
            if (!everyAgentShotWettest) {
                int idx = 0;
                for (Agent a : players.get(1).agents) {
                    Action action = new Action(ActionType.SHOOT);
                    action.setAgentId(players.get(0).agents.get((idx++) % players.get(0).getLiveAgentCount()).getId());
                    a.setCombatAction(action);
                }
            }
        } else if (leagueLevel == 3) {
            boolean objectiveComplete = league3ObjectiveComplete();
            if (!objectiveComplete) {
                for (Agent a : players.get(1).agents) {
                    Action action = new Action(ActionType.SHOOT);
                    int targetId = a.id < 5 ? 1 : 2;
                    action.setAgentId(targetId);
                    a.setCombatAction(action);
                }
            }
        } else if (leagueLevel == 4) {
            if (league4ObjectiveComplete()) {
                return;
            }

            boolean anyShoot = players.get(0).agents.stream().anyMatch(a -> a.hasShootAction());
            boolean noBombs = players.get(0).agents.isEmpty() || players.get(0).agents.get(0).getBalloons() == 0;
            boolean noSuicide = players.get(0).getLiveAgentCount() == 2;
            if (anyShoot || !noSuicide || noBombs) {
                List<Agent> executioners = new ArrayList<>(players.get(1).agents);
                Collections.shuffle(executioners, random);

                int eId = 0;
                for (Agent a : executioners) {
                    Action action = new Action(ActionType.SHOOT);
                    int targetId = (eId % 2) + 1;
                    action.setAgentId(targetId);
                    a.setCombatAction(action);
                    eId++;
                    if (eId >= 2) {
                        break;
                    }
                }
            }
        }

    }

    public boolean league4ObjectiveComplete() {
        return balloonKillOrder.stream().flatMap(s -> s.stream()).allMatch(a -> a.getWetness() >= 100);
    }

    public boolean league1ObjectiveComplete() {
        Agent agent1 = players.get(0).agents.get(0);
        Agent agent2 = players.get(0).agents.get(1);

        return (agent1.getPosition().equals(new Coord(6, 3))
            && agent2.getPosition().equals(new Coord(6, 1)))
            || (agent1.getPosition().equals(new Coord(6, 1))
                && agent2.getPosition().equals(new Coord(6, 3)));
    }

}
