package com.codingame.game.action;

import com.codingame.game.grid.Coord;

public class Action {
    final ActionType type;
    private Coord coord;
    private int agentId;

    private String message;

    public Action(ActionType type) {
        this.type = type;
    }

    public ActionType getType() {
        return type;
    }

    public int getAgentId() {
        return agentId;
    }

    public boolean isMove() {
        return this.getType() == ActionType.MOVE;
    }

    public void setAgentId(int agentId) {
        this.agentId = agentId;
    }

    public Coord getCoord() {
        return coord;
    }

    public void setCoord(Coord coord) {
        this.coord = coord;
    }

    public String getMessage() {
        return message;
    }

    public void setMessage(String message) {
        this.message = message;
    }

    @Override
    public String toString() {
        return "Action [type=" + type + ", coord=" + coord + ", amount=" + agentId + "]";
    }

    public boolean isMessage() {
        return type == ActionType.MESSAGE;
    }

    public boolean isHunkerDownAction() {
        return type == ActionType.HUNKER_DOWN;
    }

    public boolean isShootAction() {
        return type == ActionType.SHOOT;
    }

    public boolean isThrowAction() {
        return type == ActionType.THROW;
    }

}