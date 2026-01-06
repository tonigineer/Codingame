package com.codingame.game;

import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import java.util.stream.Stream;

import com.codingame.game.action.Action;
import com.codingame.game.grid.Coord;
import com.codingame.game.grid.Positionable;

public class Agent implements Positionable, Comparable<Agent> {
    int maxCooldown;
    int soakingPower;
    int optimalRange;
    int balloons;

    private boolean hunkered;
    private Coord position;
    private int wetness;

    private Action moveAction;
    private Action combatAction;
    private String message;

    Player owner;

    int id;
    int movementInterruptedAt;
    AgentClass agentClass;
    List<Coord> intendedPath;
    public int shootingId = -1;
    public boolean dying;
    public int cooldown;

    public Agent(int id, AgentClass agentClass) {
        this.id = id;
        this.balloons = agentClass.balloons;
        this.soakingPower = agentClass.soakingPower;
        this.optimalRange = agentClass.optimalRange;
        this.maxCooldown = agentClass.cooldown;
        this.agentClass = agentClass;
        this.intendedPath = new ArrayList<>();
    }

    public void reset() {
        this.message = null;
        this.moveAction = null;
        this.combatAction = null;
        this.movementInterruptedAt = -1;
        this.hunkered = false;
        this.shootingId = -1;
        this.cooldown = Math.max(0, cooldown - 1);
    }

    public List<Action> getActions() {
        return Stream.of(moveAction, combatAction)
            .filter(Objects::nonNull)
            .toList();
    }

    public int getBalloons() {
        return balloons;
    }

    public int getId() {
        return id;
    }

    public List<Coord> getIntendedPath() {
        return intendedPath;
    }

    public String getMessage() {
        return message;
    }

    public Action getMoveAction() {
        return moveAction;
    }

    public int getOptimalRange() {
        return optimalRange;
    }

    public Coord getPosition() {
        return position;
    }

    public int getSoakingPower() {
        return soakingPower;
    }

    public Action getCombatAction() {
        return combatAction;
    }

    public int getWetness() {
        return wetness;
    }

    public boolean gotBlocked() {
        return movementInterruptedAt > -1;
    }

    public boolean hasThrowAction() {
        return combatAction != null && combatAction.isThrowAction();
    }

    public boolean hasHunkerDownAction() {
        return combatAction != null && combatAction.isHunkerDownAction();
    }

    public boolean hasMoveAction() {
        return moveAction != null;
    }

    public boolean hasShootAction() {
        return combatAction != null && combatAction.isShootAction();
    }

    public boolean isHunkered() {
        return hunkered;
    }

    public void setHunkered(boolean b) {
        this.hunkered = b;
    }

    public void setIntendedPath(List<Coord> path) {
        this.intendedPath = path;
    }

    public void setMessage(String message) {
        this.message = message;

    }

    public void setMoveAction(Action moveAction) {
        this.moveAction = moveAction;
    }

    public void setMovementInterrupted(int currentStep) {
        this.movementInterruptedAt = currentStep;
    }

    public void setPosition(Coord position) {
        this.position = position;
    }

    public void setCombatAction(Action combatAction) {
        this.combatAction = combatAction;
    }

    public void setWetness(int wetness) {
        this.wetness = wetness;
    }

    @Override
    public double getDistanceMultiplier() {
        if (Game.WETNESS_AFFECTS_DISTANCE) {
            return wetness >= 50 ? 2 : 1;
        }
        return 1;
    }

    public void removeBalloon() {
        balloons--;        
    }

    @Override
    public int compareTo(Agent other) {
        return Integer.compare(this.id, other.id);
    }

}
