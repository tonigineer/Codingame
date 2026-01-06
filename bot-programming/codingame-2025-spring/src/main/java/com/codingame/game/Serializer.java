package com.codingame.game;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;
import java.util.stream.Stream;

import com.codingame.event.EventData;
import com.codingame.game.grid.Coord;
import com.codingame.game.grid.Grid;

public class Serializer {
    public static final String MAIN_SEPARATOR = "|";

    static public <T> String serialize(List<T> list, String separator) {
        return list.stream().map(String::valueOf).collect(Collectors.joining(separator));
    }

    static public String serialize(int[] intArray) {
        return Arrays.stream(intArray).mapToObj(String::valueOf).collect(Collectors.joining(" "));
    }

    static public String serialize(boolean[] boolArray) {
        List<String> strs = new ArrayList<>(boolArray.length);
        for (boolean b : boolArray) {
            strs.add(b ? "1" : "0");
        }
        return strs.stream().collect(Collectors.joining(" "));
    }

    static public String join(Object... args) {
        return Stream.of(args)
            .map(String::valueOf)
            .collect(Collectors.joining(" "));
    }

    public static String serializeGlobalData(Game game) {
        List<Object> lines = new ArrayList<>();

        lines.add(game.leagueLevel);
        lines.add(game.grid.width);
        lines.add(game.grid.height);
        List<Integer> tileTypes = new ArrayList<>(game.grid.width * game.grid.height);
        for (int y = 0; y < game.grid.height; ++y) {
            for (int x = 0; x < game.grid.width; ++x) {
                Coord c = new Coord(x, y);
                int type = game.grid.get(c).getType();
                tileTypes.add(type);
            }
        }
        lines.add(tileTypes.stream().map(String::valueOf).collect(Collectors.joining("")));

        int agentCount = 0;
        for (Player p : game.players) {
            agentCount += p.agents.size();
        }
        lines.add(agentCount);
        for (Player p : game.players) {
            for (Agent a : p.agents) {
                lines.add(a.id);
                lines.add(a.getPosition().getX());
                lines.add(a.getPosition().getY());

                lines.add(a.maxCooldown);
                lines.add(a.optimalRange);
                lines.add(a.soakingPower);
                
                lines.add(a.owner.getIndex());
                lines.add(a.getBalloons());
                lines.add(a.getWetness());
            }
        }

        if (game.leagueLevel == 3) {
            // Send objective markers
            for (Coord c : game.tutorialManager.getRunAndGunObjectiveCoords()) {
                lines.add(c.toIntString());
            }
        }

        return lines.stream()
            .map(String::valueOf)
            .collect(Collectors.joining(MAIN_SEPARATOR));
    }

    public static String serializeFrameData(Game game) {
        List<Object> lines = new ArrayList<>();

        lines.add(game.getViewerEvents().size());
        game.getViewerEvents().stream()
            .flatMap(
                e -> Stream.of(
                    e.type,
                    e.animData.start,
                    e.animData.end,
                    e.coord == null ? "" : e.coord.toIntString(),
                    e.target == null ? "" : e.target.toIntString(),
                    serialize(e.params)
                )
            )
            .forEach(lines::add);

        // Send control zones
        List<Integer> ints = new ArrayList<>(game.grid.height * game.grid.width);
        for (int y = 0; y < game.grid.height; ++y) {
            for (int x = 0; x < game.grid.width; ++x) {
                Coord c = new Coord(x, y);
                if (game.controlZone.get(0).contains(c)) {
                    ints.add(0);
                } else
                    if (game.controlZone.get(1).contains(c)) {
                        ints.add(1);
                    } else {
                        ints.add(2);
                    }
            }
        }
        lines.add(serialize(ints, ""));

        // Player scores
        for (Player p : game.players) {
            lines.add(p.points);
        }

        // Agent messages
        List<Agent> messagers = game.allAgentStream()
            .filter(a -> a.getMessage() != null)
            .toList();
        lines.add(messagers.size());
        messagers.forEach(a -> {
            lines.add(a.id);
            lines.add(a.getMessage().replaceAll("\\|", "∣"));
        });

        return lines.stream()
            .map(String::valueOf)
            .collect(Collectors.joining(MAIN_SEPARATOR));
    }

    private static boolean isCoordOnZoneBorder(Coord c, Grid grid, List<Coord> playerZone) {
        List<Coord> neighs = grid.getNeighbours(c, Grid.ADJACENCY_8);
        if (neighs.size() < 8) {
            return true;
        }
        return neighs.stream().anyMatch(n -> !playerZone.contains(n));
    }

    public static List<String> serializeGlobalInfoFor(Player player, Game game) {
        List<Object> lines = new ArrayList<>();

        // Self id
        lines.add(String.valueOf(player.getIndex()));

        // Agents
        List<Agent> allAgents = game.getAllAgents();
        lines.add(allAgents.size());
        for (Agent a : allAgents) {
            lines.add(
                Stream
                    .of(
                        a.id,
                        a.owner.getIndex(),
                        a.maxCooldown,
                        a.getOptimalRange(),
                        a.getSoakingPower(),
                        a.getBalloons()
                    )
                    .map(String::valueOf)
                    .collect(Collectors.joining(" "))
            );
        }

        // Map
        lines.add(game.grid.width + " " + game.grid.height);
        for (int y = 0; y < game.grid.height; ++y) {
            List<String> row = new ArrayList<>(game.grid.width);
            for (int x = 0; x < game.grid.width; ++x) {
                Coord c = new Coord(x, y);
                int type = game.grid.get(c).getType();
                row.add(join(x, y, type));
            }
            lines.add(serialize(row, " "));
        }

        return lines.stream()
            .map(String::valueOf)
            .collect(Collectors.toList());
    }

    public static List<String> serializeFrameInfoFor(Player player, Game game) {
        List<Object> lines = new ArrayList<>();
        List<Agent> allAgents = game.getAllAgents();
        lines.add(allAgents.size());
        for (Agent a : allAgents) {
            lines.add(
                Stream
                    .of(
                        a.id,
                        a.getPosition().getX(),
                        a.getPosition().getY(),
                        a.cooldown,
                        a.getBalloons(),
                        a.getWetness()
                    )
                    .map(String::valueOf)
                    .collect(Collectors.joining(" "))
            );
        }
        lines.add(player.agents.size());

        return lines.stream()
            .map(String::valueOf)
            .collect(Collectors.toList());
    }

    public static String serializeEventCoords(EventData e) {
        return Stream.of(e.coord, e.target).filter(c -> !Objects.isNull(c))
            .map(
                coord -> coord.toIntString()
            ).collect(
                Collectors.joining("_")
            );
    }

}
