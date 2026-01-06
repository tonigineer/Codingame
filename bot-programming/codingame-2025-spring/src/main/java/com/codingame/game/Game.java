
package com.codingame.game;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Random;
import java.util.Set;
import java.util.stream.Collectors;
import java.util.stream.Stream;

import com.codingame.event.Animation;
import com.codingame.event.EventData;
import com.codingame.game.action.Action;
import com.codingame.game.grid.Coord;
import com.codingame.game.grid.Grid;
import com.codingame.game.grid.GridMaker;
import com.codingame.game.grid.Tile;
import com.codingame.game.pathfinding.PathFinder;
import com.codingame.game.pathfinding.PathFinder.PathFinderResult;
import com.codingame.gameengine.core.GameManager;
import com.codingame.gameengine.core.MultiplayerGameManager;
import com.codingame.gameengine.module.endscreen.EndScreenModule;
import com.google.inject.Inject;
import com.google.inject.Singleton;

//TODO: player scores in input 

@Singleton
public class Game {

    public final static boolean ALLOW_DOUBLE_MOVE = false;
    public final static boolean WETNESS_AFFECTS_DISTANCE = true;
    public final static boolean COLLISIONS = true;
    public final static int THROW_DAMAGE = 30; // TODO: 24 or 32 ?
    public final static int THROW_DISTANCE_MAX = 4;
    public final static int MAX_POINT_DIFF = 600;

    static boolean ONE_PLAYER_MODE;
    static boolean CONTROL_ZONES;
    static boolean ALLOW_THROW;
    static boolean ALLOW_SHOOT;
    static boolean ALLOW_MOVE;
    static boolean ALLOW_HUNKER;

    @Inject private MultiplayerGameManager<Player> gameManager;
    @Inject private EndScreenModule endScreenModule;
    @Inject private Animation animation;
    @Inject private PathFinder pathfinder;
    @Inject TutorialManager tutorialManager;

    List<Player> players;
    Random random;
    Grid grid;
    List<List<Coord>> controlZone;
    int teamlessCount;
    int leagueLevel;
    boolean inTutorial;

    public void init() {
        ONE_PLAYER_MODE = false;
        CONTROL_ZONES = true;
        ALLOW_THROW = true;
        ALLOW_SHOOT = true;
        ALLOW_MOVE = true;
        ALLOW_HUNKER = true;

        players = gameManager.getPlayers();
        random = gameManager.getRandom();
        teamlessCount = 0;

        this.leagueLevel = gameManager.getLeagueLevel();

        inTutorial = tutorialManager.initTutorial(random, this.leagueLevel);
        if (!inTutorial) {
            initGrid();
            initPlayers();
        }

        initControlZones();
    }

    private void initControlZones() {
        controlZone = new ArrayList<>(players.size());
        for (Player p : players) {
            controlZone.add(new ArrayList<>());
        }
        updateControlZones();
    }

    private void initPlayers() {
        int agentId = 1;
        List<AgentClass> agentClasses = new ArrayList<>(Arrays.asList(AgentClass.values()));

        for (Player p : players) {
            int agentIdx = 0;
            for (Coord c : grid.spawns) {
                AgentClass agentClass = agentClasses.get(agentIdx++);
                Agent a = new Agent(agentId++, agentClass);
                if (p.getIndex() == 1) {
                    c = grid.opposite(c);

                }
                p.agents.add(a);
                a.owner = p;
                a.setPosition(c);
            }

        }
    }

    private void initGrid() {
        this.grid = GridMaker.initGrid(random);
        pathfinder.setGrid(grid);
    }

    public void resetGameTurnData() {
        animation.reset();

        List<Agent> deadAgents = new ArrayList<>();
        allAgentStream()
            .filter(a -> a.dying)
            .forEach(a -> {
                deadAgents.add(a);
            });

        for (Agent a : deadAgents) {
            a.owner.agents.remove(a);
            launchDeathEvent(a);
        }

        players.forEach(Player::reset);
        controlZone.stream().forEach(List::clear);
    }

    public void performGameUpdate(int turn) {
        if (inTutorial) {
            tutorialManager.changeTutorialBossActions(turn);
        }

        if (!isCinematicFrame()) {
            doMoves();
            animation.catchUp();
            doHunkers();
            animation.catchUp();
            doShoots();
            doThrows();
            animation.catchUp();
            doDeaths();
            updateControlZones();
            scorePoints();
        }

        computeEvents();
        if (isGameOver()) {
            gameManager.endGame();
        }
        countTeamless();
    }

    private void countTeamless() {
        this.teamlessCount = (int) players.stream()
            .filter(p -> p.getLiveAgentCount() == 0)
            .count();
    }

    private void scorePoints() {
        for (Player p : players) {
            int diff = controlZone.get(p.getIndex()).size() - controlZone.get(1 - p.getIndex()).size();
            if (diff > 0) {
                p.addScore(diff);
            }
        }

    }

    private void updateControlZones() {
        if (!CONTROL_ZONES) {
            return;
        }
        for (Coord c : grid.getCoords()) {
            List<Agent> liveAgents = allAgentStream()
                .filter(a -> !a.dying)
                .toList();

            List<Agent> closestAgents = grid.getClosestTargets(c, liveAgents);

            if (closestAgents.isEmpty()) {
                continue;
            }
            if (closestAgents.stream().map(a -> a.owner).distinct().count() == 1) {
                Player owner = closestAgents.get(0).owner;
                controlZone.get(owner.getIndex()).add(c);
            }
        }
    }

    private void doThrows() {
        allAgentStream()
            .filter(a -> a.hasThrowAction())
            .forEach(agent -> {
                if (!ALLOW_THROW) {
                    reportPlayerError(
                        "%s bombs are not allowed".formatted(agent.owner.getNicknameToken())
                    );
                    return;
                }

                Action bombThrow = agent.getCombatAction();
                Coord target = bombThrow.getCoord();
                if (!grid.get(target).isValid()) {
                    reportPlayerError(
                        "%s cannot throw splash bombs to %s.".formatted(agent.owner.getNicknameToken(), target.toIntString())
                    );
                    return;
                }
                if (target.manhattanTo(agent.getPosition()) > THROW_DISTANCE_MAX) {
                    reportPlayerError(
                        "%s cannot throw splash bomb to %s, it is too far away from agent %d."
                            .formatted(agent.owner.getNicknameToken(), target.toIntString(), agent.getId())
                    );
                    return;
                }
                if (agent.getBalloons() <= 0) {
                    reportPlayerError(
                        "%s agent %d has no splash bomb to throw.".formatted(agent.owner.getNicknameToken(), agent.getId())
                    );
                    return;
                }
                List<Coord> affected = Stream.concat(
                    Stream.of(target),
                    grid.getNeighbours(target, Grid.ADJACENCY_8).stream()
                ).toList();

                List<Agent> gotWet = allAgentStream()
                    .filter(other -> affected.contains(other.getPosition()))
                    .toList();
                gotWet.forEach(other -> {
                    other.setWetness(other.getWetness() + THROW_DAMAGE);
                });
                launchThrowEvent(agent, target, gotWet);
                agent.removeBalloon();
            });
    }

    private void launchThrowEvent(Agent agent, Coord target, List<Agent> gotWet) {
        double distance = agent.getPosition().euclideanTo(target);
        EventData e = new EventData();
        e.type = EventData.THROW;
        e.coord = agent.getPosition();
        e.target = target;
        int throwTime = (int) (Animation.WHOLE * distance / 3);
        int boomTime = Animation.THIRD * 2;
        int pIdx = 0;
        e.params = new int[3 + gotWet.size()];
        e.params[pIdx++] = agent.getId();
        e.params[pIdx++] = THROW_DAMAGE;
        e.params[pIdx++] = (int) (((double) throwTime / (throwTime + boomTime)) * 10_000);
        for (int i = 0; i < gotWet.size(); ++i) {
            e.params[i + pIdx] = gotWet.get(i).getId();
        }
        animation.startAnim(e, throwTime + boomTime);

    }

    private void doDeaths() {
        allAgentStream()
            .filter(a -> a.getWetness() >= 100)
            .forEach(a -> {
                a.dying = true;
            });
    }

    private void doHunkers() {
        allAgentStream()
            .filter(a -> a.hasHunkerDownAction())
            .forEach(a -> {
                if (ALLOW_HUNKER) {
                    a.setHunkered(true);
                    launchHunkerEvent(a);
                }
            });
    }

    private void reportPlayerError(String message) {
        gameManager.addToGameSummary(
            GameManager.formatErrorMessage(message)
        );
    }

    private void doShoots() {
        allAgentStream()
            .filter(a -> a.hasShootAction())
            .forEach(a -> {
                Action shootAction = a.getCombatAction();
                if (!ALLOW_SHOOT) {
                    reportPlayerError(
                        "%s shooting is not allowed".formatted(a.owner.getNicknameToken())
                    );
                    return;
                }
                if (a.cooldown > 0) {
                    reportPlayerError(
                        "%s agent id %d cannot shoot yet. Cooldown = %d".formatted(a.owner.getNicknameToken(), a.id, a.cooldown)
                    );
                    return;
                }
                int targetId = shootAction.getAgentId();
                Agent target = getAgentById(targetId);
                if (target == null) {
                    reportPlayerError(
                        "%s agent id %d cannot shoot agent id %d, they do not exist".formatted(a.owner.getNicknameToken(), a.id, targetId)
                    );
                    return;
                } else if (target == a) {
                    reportPlayerError(
                        "%s agent id %d cannot shoot themselves".formatted(a.owner.getNicknameToken(), targetId)
                    );
                    return;
                }

                double rangeModifier = getRangeModifier(target, a);

                if (rangeModifier == 0) {
                    reportPlayerError(
                        "%s agent id %d cannot shoot id %d, they are too far away".formatted(a.owner.getNicknameToken(), a.id, targetId)
                    );
                    return;
                }

                double coverModifier = getCoverModifier(target, a);
                double hunkerModifierBonus = getHunkeredModifierBonus(target);

                int damage = (int) Math.round(a.getSoakingPower() * rangeModifier * (coverModifier - hunkerModifierBonus));
                target.setWetness(target.getWetness() + damage);
                a.shootingId = target.id;
                launchShootEvent(target, a, rangeModifier, damage);
                a.cooldown = a.maxCooldown + 1;
            });
    }

    private double getHunkeredModifierBonus(Agent target) {
        if (target.isHunkered()) {
            return 0.25;
        }
        return 0;
    }

    private double getCoverModifier(Agent target, Agent shooter) {
        double dx = target.getPosition().getX() - shooter.getPosition().getX();
        double dy = target.getPosition().getY() - shooter.getPosition().getY();
        double bestModifier = 1;

        for (double[] d : new double[][] { { dx, 0 }, { 0, dy } }) {
            if (Math.abs(d[0]) > 1 || Math.abs(d[1]) > 1) {
                int adjX = (int) -Math.signum(d[0]);
                int adjY = (int) -Math.signum(d[1]);
                Coord coverPos = new Coord(
                    target.getPosition().getX() + adjX,
                    target.getPosition().getY() + adjY
                );
                if (coverPos.chebyshevTo(shooter.getPosition()) > 1) {
                    Tile t = grid.get(coverPos);
                    bestModifier = Math.min(bestModifier, t.getCoverModifier());
                }
            }
        }

        return bestModifier;
    }

    private double getRangeModifier(Agent target, Agent shooter) {
        double distanceToTarget = target.getPosition().manhattanTo(shooter.getPosition());
        if (distanceToTarget <= shooter.getOptimalRange()) {
            return 1;
        }
        if (distanceToTarget <= shooter.getOptimalRange() * 2) {
            return 0.5;
        }
        return 0;
    }

    Agent getAgentById(int targetId) {
        // a map would be nice
        return allAgentStream().filter(a -> a.getId() == targetId).findFirst().orElse(null);
    }

    private void launchMoveEvent(Agent a, Coord from, Coord to) {
        EventData e = new EventData();
        e.type = EventData.MOVE;
        e.coord = from;
        e.target = to;
        e.params = new int[] {
            a.getId()
        };
        animation.startAnim(e, Animation.HALF);
    }

    private void launchMoveConflictEvent(Move move) {
        EventData e = new EventData();
        e.type = EventData.MOVE_CONFLICT;
        e.coord = move.from;
        e.target = move.to;
        e.params = new int[] {
            move.agent.getId()
        };
        animation.startAnim(e, Animation.THIRD);
    }

    private void launchDeathEvent(Agent a) {
        EventData e = new EventData();
        e.type = EventData.DEATH;
        e.coord = a.getPosition();
        e.params = new int[] {
            a.getId()
        };
        animation.startAnim(e, Animation.WHOLE);
    }

    private void launchHunkerEvent(Agent agent) {
        EventData e = new EventData();
        e.type = EventData.HUNKER;
        e.params = new int[] {
            agent.getId()
        };
        animation.startAnim(e, Animation.THIRD);
    }

    private void launchShootEvent(Agent target, Agent shooter, double rangeModifier, int damage) {
        double distance = shooter.getPosition().euclideanTo(target.getPosition());
        int stretchTime = (int) (Animation.HALF * distance / 7);
        int holdTime = Animation.HALF;

        EventData e = new EventData();
        e.type = EventData.SHOOT;
        e.coord = shooter.getPosition();
        e.target = target.getPosition();
        e.params = new int[] {
            shooter.getId(),
            target.getId(),
            (int) (rangeModifier * 2), //0,1,2
            damage,
            (int) (((double) stretchTime / (stretchTime + holdTime)) * 10_000)
        };
        animation.startAnim(e, stretchTime + holdTime);
    }

    record Move(
        Agent agent,
        Coord from,
        Coord to
    ) {
    }

    private void doMoves() {
        List<Move> moves = new ArrayList<>();
        Set<Agent> staticAgents = allAgentStream()
            .collect(Collectors.toSet());
        List<Coord> currentlyOccupied = staticAgents.stream().map(Agent::getPosition).toList();

        allAgentStream()
            .filter(a -> a.hasMoveAction())
            .forEach(a -> {
                if (!ALLOW_MOVE) {
                    reportPlayerError(
                        "%s moving is not allowed".formatted(a.owner.getNicknameToken(), a.getId())
                    );
                    return;
                }
                PathFinderResult res = pathfinder
                    .restrict(currentlyOccupied)
                    .from(a.getPosition())
                    .to(a.getMoveAction().getCoord())
                    .findPath();
                if (res.hasNextCoord()) {
                    moves.add(new Move(a, a.getPosition(), res.getNextCoord()));
                    staticAgents.remove(a);
                } else if (res.hasNoPath()) {
                    reportPlayerError(
                        "%s id=%d cannot move to %s".formatted(a.owner.getNicknameToken(), a.getId(), a.getMoveAction().getCoord().toIntString())
                    );
                }
            });

        if (COLLISIONS) {
            List<Move> movesToCancel = new ArrayList<>();

            // Cancel collisions with static agents
            moves.stream()
                .filter(m -> collidesWithStaticAgent(m, staticAgents))
                .forEach(m -> {
                    movesToCancel.add(m);
                });

            movesToCancel.forEach(
                m -> {
                    moves.remove(m);
                    launchMoveConflictEvent(m);
                }
            );

            // Detect move conflicts
            for (Move m : moves) {
                moves.stream()
                    .filter(o -> o != m)
                    .filter(o -> o.to.equals(m.to) || o.to.equals(m.from) && m.to.equals(o.from))
                    .forEach(o -> {
                        movesToCancel.add(o);
                    });
            }
            while (!movesToCancel.isEmpty()) {
                // Cancel conflicting moves
                List<Move> nextMovesToCancel = new ArrayList<>();
                moves.removeAll(movesToCancel);

                // Detect new conflicts
                for (Move cancelled : movesToCancel) {
                    launchMoveConflictEvent(cancelled);
                    moves.stream()
                        .filter(o -> o != cancelled)
                        .filter(o -> o.to.equals(cancelled.from))
                        .forEach(o -> {
                            nextMovesToCancel.add(o);
                        });
                }
                movesToCancel.clear();
                movesToCancel.addAll(nextMovesToCancel);
            }
        }
        for (Move m : moves) {
            m.agent.setPosition(m.to);
            launchMoveEvent(m.agent, m.from, m.to);
        }
    }

    private boolean collidesWithStaticAgent(Move move, Set<Agent> staticAgents) {
        return staticAgents.stream()
            .filter(a -> a.getPosition().equals(move.to))
            .findFirst()
            .isPresent();
    }

    List<Agent> getAllAgents() {
        return allAgentStream().toList();
    }

    Stream<Agent> allAgentStream() {
        return players.stream().flatMap(p -> p.agents.stream());
    }

    public boolean isGameOver() {
        if (leagueLevel == 1) {
            return tutorialManager.league1ObjectiveComplete();
        } else if (leagueLevel == 4) {
            if (tutorialManager.league4ObjectiveComplete()) {
                return true;
            }
        }
        boolean p0win = players.get(0).points > players.get(1).points + MAX_POINT_DIFF;
        boolean p1win = players.get(1).points > players.get(0).points + MAX_POINT_DIFF;

        return p0win || p1win || teamlessCount > 0 || gameManager.getActivePlayers().size() < 2;
    }

    public void onEnd() {
        String[] scoreTexts = new String[players.size()];

        if (inTutorial) {
            tutorialManager.handleEnd(scoreTexts);
        } else {
            boolean bothDead = teamlessCount == players.size();
            for (Player p : players) {

                if (!p.isActive() || (p.getLiveAgentCount() == 0 && !bothDead)) {
                    p.setScore(-1);
                    scoreTexts[p.getIndex()] = "-";
                } else {
                    p.setScore(p.points);
                    scoreTexts[p.getIndex()] = p.points + " point" + (p.points > 1 ? "s" : "");
                }
            }
        }

        int[] scores = players.stream().mapToInt(Player::getScore).toArray();
        endScreenModule.setScores(scores, scoreTexts);

        computeMetadata();

    }

    private void computeMetadata() {

    }

    public static String getExpected(String command) {
        return "MOVE x y | SHOOT id | THROW x y | HUNKER_DOWN | MESSAGE text";
    }

    public List<EventData> getViewerEvents() {
        return animation.getViewerEvents();
    }

    private void computeEvents() {
        int minTime = 500;

        animation.catchUp();

        int frameTime = Math.max(
            animation.getFrameTime(),
            minTime
        );
        gameManager.setFrameDuration(frameTime);
    }

    public boolean isCinematicFrame() {
        return !ONE_PLAYER_MODE && teamlessCount > 0;
    }

    public boolean shouldSkipPlayerTurn(Player player) {
        if (inTutorial) {
            return player.getIndex() == 1 || isCinematicFrame();
        }
        return isCinematicFrame();
    }

}
