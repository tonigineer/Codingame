
package com.codingame.game.grid;

import java.util.ArrayList;
import java.util.Collections;
import java.util.HashSet;
import java.util.LinkedList;
import java.util.List;
import java.util.Optional;
import java.util.Queue;
import java.util.Random;
import java.util.Set;
import java.util.stream.Collectors;

import com.google.inject.Singleton;

@Singleton
public class GridMaker {
    private static final int GRID_W_RATIO = 2;
    public static final int MIN_SPAWN_COUNT = 3;
    public static final int MAX_SPAWN_COUNT = 5;
    public static final int MIN_HEIGHT = 6;
    public static final int MAX_HEIGHT = 10;

    public static Grid initEmpty(int w, int h) {
        Grid grid = new Grid(w, h, false);
        for (int y = 0; y < h; ++y) {
            for (int x = 0; x < w; ++x) {
                grid.get(x, y).setType(Tile.TYPE_FLOOR);
            }
        }
        return grid;
    }
    
    public static Grid initGrid(Random random) {
        int h = random.nextInt(MIN_HEIGHT, MAX_HEIGHT + 1);
        int w = h * GRID_W_RATIO;

        boolean ySym = random.nextBoolean() || random.nextBoolean();
        Grid grid = new Grid(w, h, ySym);

        List<Coord> allCoords = new ArrayList<>(grid.getCoords());

        // Walls
        for (int y = 1; y < h-1; ++y) {
            for (int x = 1; x < w / 2 - 1; ++x) {
                Coord coord = new Coord(x, y);
                Coord opp = grid.opposite(coord);
                int n = random.nextInt(10);
                int type = Tile.TYPE_FLOOR;
                if (n == 0) {
                    type = Tile.TYPE_HIGH_COVER;
                } else if (n == 1) {
                    type = Tile.TYPE_LOW_COVER;
                }
                grid.get(coord).setType(type);
                grid.get(opp).setType(type);
            }
        }

        // Spawns
        List<Coord> allLeftCoords = new ArrayList<>(
            grid.getCoords().stream().filter(
                coord -> coord.getX() == 0
            ).toList()
        );

        int spawnCount = random.nextInt(MIN_SPAWN_COUNT, MAX_SPAWN_COUNT + 1);
        Collections.shuffle(allLeftCoords, random);
        if (spawnCount == 5 && random.nextBoolean()) {
            spawnCount--;
        }
        if (spawnCount == 4 && random.nextBoolean()) {
            spawnCount--;
        }

        for (int i = 0; i < spawnCount; i++) {
            Coord c = allLeftCoords.get(i);
            grid.spawns.add(c);
            grid.get(c).clear();
            grid.get(grid.opposite(c)).clear();
        }

        List<Coord> wallCoords = grid.getCoords().stream().filter(c -> grid.get(c).isCover()).toList();

        fixIslands(grid, new ArrayList<>(wallCoords), random);

        return grid;
    }
    

    private static Optional<Set<Coord>> getIslandFrom(List<Set<Coord>> islands, Coord coord) {
        return islands.stream()
            .filter(set -> set.contains(coord))
            .findFirst();
    }

    private static boolean closeIslandGap(Grid grid, List<Coord> wallCoords, List<Set<Coord>> islands) {
        List<Set<Coord>> connectingIslands = null;
        Coord bridge = null;

        for (Coord coord : wallCoords) {
            List<Coord> neighs = grid.getNeighbours(coord);
            connectingIslands = neighs.stream()
                .map(n -> getIslandFrom(islands, n))
                .filter(opt -> opt.isPresent())
                .map(opt -> opt.get())
                .distinct()
                .collect(Collectors.toList());
            if (connectingIslands.size() > 1) {
                bridge = coord;
                break;
            }
        }

        if (bridge != null) {
            final List<Set<Coord>> bridging = connectingIslands;
            Coord coord = bridge;
            Coord opposite = grid.opposite(coord);

            grid.get(coord).clear();
            grid.get(opposite).clear();

            wallCoords.remove(coord);
            wallCoords.remove(opposite);

            List<Set<Coord>> newIslands = islands.stream()
                .filter(set -> !bridging.contains(set))
                .collect(Collectors.toList());

            Set<Coord> newIsland = new HashSet<>();
            bridging.forEach(set -> newIsland.addAll(set));

            islands.clear();
            islands.addAll(newIslands);
            islands.add(newIsland);
            return true;
        }
        return false;
    }

    private static List<Set<Coord>> detectIslands(Grid grid) {
        List<Set<Coord>> islands = new ArrayList<>();
        Set<Coord> computed = new HashSet<>();
        Set<Coord> current = new HashSet<>();

        for (Coord p : grid.getCoords()) {
            if (grid.get(p).isCover()) {
                continue;
            }
            if (!computed.contains(p)) {
                Queue<Coord> fifo = new LinkedList<>();
                fifo.add(p);
                computed.add(p);

                while (!fifo.isEmpty()) {
                    Coord e = fifo.poll();
                    for (Coord delta : Grid.ADJACENCY) {
                        Coord n = e.add(delta);
                        Tile cell = grid.get(n);
                        if (cell.isValid() && !cell.isCover() && !computed.contains(n)) {
                            fifo.add(n);
                            computed.add(n);
                        }
                    }
                    current.add(e);
                }
                islands.add(new HashSet<>(current));
                current.clear();
            }
        }

        return islands;
    }

    private static void fixIslands(Grid grid, List<Coord> wallCoords, Random random) {
        Collections.shuffle(wallCoords, random);
        List<Set<Coord>> islands = detectIslands(grid);

        while (islands.size() > 1) {
            boolean closed = closeIslandGap(grid, wallCoords, islands);

            if (!closed) {
                Coord aWallAdjToFree = findWallAdjacentToFreeSpace(wallCoords, grid);
                if (aWallAdjToFree != null) {
                    Coord coord = aWallAdjToFree;
                    Coord opposite = grid.opposite(coord);

                    grid.get(coord).clear();
                    grid.get(opposite).clear();

                    wallCoords.remove(coord);
                    wallCoords.remove(opposite);
                }
                islands = detectIslands(grid);
            }
        }

    }

    private static Coord findWallAdjacentToFreeSpace(List<Coord> wallCoords, Grid grid) {
        for (Coord c : wallCoords) {
            List<Coord> neighs = grid.getNeighbours(c);
            Optional<Coord> free = neighs.stream().filter(n -> !grid.get(n).isCover()).findAny();
            if (free.isPresent()) {
                return c;
            }
        }
        return null;
    }

}
