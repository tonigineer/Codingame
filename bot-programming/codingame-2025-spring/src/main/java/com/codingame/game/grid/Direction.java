package com.codingame.game.grid;

public enum Direction {
    NORTH(0, -1, "N"),
    EAST(1, 0, "E"),
    SOUTH(0, 1, "S"),
    WEST(-1, 0, "W"),
    UNSET(0, 0, "X");

    public Coord coord;
    public String alias;

    Direction(int x, int y, String alias) {
        this.coord = new Coord(x, y);
        this.alias = alias;
    }

    @Override
    public String toString() {
        return alias;
    }
    
    

    public static Direction fromAlias(String alias) {
        switch (alias) {
        case "N":
            return NORTH;
        case "E":
            return EAST;
        case "S":
            return SOUTH;
        case "W":
            return WEST;
        }
        throw new RuntimeException(alias + " is not a direction alias");
    }
}
