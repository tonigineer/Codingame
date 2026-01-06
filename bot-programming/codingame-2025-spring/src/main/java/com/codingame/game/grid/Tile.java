package com.codingame.game.grid;

public class Tile {
    public static final Tile NO_TILE = new Tile(new Coord(-1, -1), -1);

    public static int TYPE_FLOOR = 0;
    public static int TYPE_LOW_COVER = 1;
    public static int TYPE_HIGH_COVER = 2;

    private int type;
    Coord coord;

    public Tile(Coord coord) {
        this.coord = coord;
    }

    public Tile(Coord coord, int type) {
        this.coord = coord;
        this.setType(type);
    }

    public void setType(int type) {
        this.type = type;
    }

    public int getType() {
        return type;
    }

    public boolean isCover() {
        return type != TYPE_FLOOR;
    }

    public double getCoverModifier() {
        if (type == TYPE_LOW_COVER) {
            return 0.5;
        } else if (type == TYPE_HIGH_COVER) {
            return 0.25;
        }
        return 1;
    }

    public void clear() {
        type = TYPE_FLOOR;
    }

    public boolean isValid() {
        return this != NO_TILE;
    }

}
