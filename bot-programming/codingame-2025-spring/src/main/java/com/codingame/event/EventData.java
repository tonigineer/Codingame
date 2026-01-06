package com.codingame.event;

import com.codingame.game.grid.Coord;

public class EventData {
    public static final int MOVE = 0;
    public static final int HUNKER = 1;
    public static final int SHOOT = 2;
    public static final int DEATH = 3;
    public static final int THROW = 4;
    public static final int MOVE_CONFLICT = 5;

    public int type;
    public AnimationData animData;

    public Coord coord, target;
    public int[] params;

    public EventData() {

    }

}
