package com.codingame.game.grid;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;

public class Grid {
    public static final Coord[] ADJACENCY = new Coord[] { Direction.NORTH.coord, Direction.EAST.coord, Direction.SOUTH.coord, Direction.WEST.coord };

    public static final Coord[] ADJACENCY_8 = new Coord[] {
        Direction.NORTH.coord,
        Direction.EAST.coord,
        Direction.SOUTH.coord,
        Direction.WEST.coord,
        new Coord(-1, -1),
        new Coord(1, 1),
        new Coord(1, -1),
        new Coord(-1, 1)
    };

    public int width, height;
    public LinkedHashMap<Coord, Tile> cells;
    boolean ySymetry;
    public List<Coord> spawns;

    public Grid(int width, int height) {
        this(width, height, false);
    }

    public Grid(int width, int height, boolean ySymetry) {
        this.width = width;
        this.height = height;
        this.ySymetry = ySymetry;
        spawns = new ArrayList<>();

        cells = new LinkedHashMap<>();

        for (int y = 0; y < height; ++y) {
            for (int x = 0; x < width; ++x) {
                Coord coord = new Coord(x, y);
                Tile cell = new Tile(coord);
                cells.put(coord, cell);
            }
        }
    }

    public Tile get(int x, int y) {
        return cells.getOrDefault(new Coord(x, y), Tile.NO_TILE);
    }

    public List<Coord> getNeighbours(Coord pos, Coord[] adjacency) {
        List<Coord> neighs = new ArrayList<>();
        for (Coord delta : adjacency) {
            Coord n = new Coord(pos.getX() + delta.getX(), pos.getY() + delta.getY());
            if (get(n) != Tile.NO_TILE) {
                neighs.add(n);
            }
        }
        return neighs;
    }

    public List<Coord> getNeighbours(Coord pos) {
        return getNeighbours(pos, ADJACENCY);
    }

    public Tile get(Coord n) {
        return get(n.getX(), n.getY());
    }

    public <T extends Positionable> List<T> getClosestTargets(Coord from, List<T> targets) {
        List<T> closest = new ArrayList<>();
        int closestBy = 0;
        for (T targ : targets) {
            Coord neigh = targ.getPosition();
            double mult = targ.getDistanceMultiplier();
            int distance = (int) (from.manhattanTo(neigh) * mult);
            if (closest.isEmpty() || closestBy > distance) {
                closest.clear();
                closest.add(targ);
                closestBy = distance;
            } else if (!closest.isEmpty() && closestBy == distance) {
                closest.add(targ);
            }
        }
        return closest;
    }

    public List<Coord> getCoords() {
        return cells.keySet().stream().toList();
    }

    public Coord opposite(Coord c) {
        return new Coord(width - c.x - 1, ySymetry ? (height - c.y - 1) : c.y);
    }

    public boolean isYSymetric() {
        return ySymetry;
    }
}
