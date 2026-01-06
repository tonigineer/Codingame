package com.codingame.game;

public enum AgentClass {

    GUNNER(1, 16, 4, 1),
    SNIPER(5, 24, 6, 0),
    BOMBER(2, 8, 2, 3),
    ASSAULT(2, 16, 4, 2),
    BERSERKER(5, 32, 2, 1);

    final int cooldown;
    final int soakingPower;
    final int optimalRange;
    final int balloons;

    private AgentClass(int speed, int soakingPower, int optimalRange, int balloons) {
        this.cooldown = speed;
        this.soakingPower = soakingPower;
        this.optimalRange = optimalRange;
        this.balloons = balloons;
    }

    public int getCooldown() {
        return cooldown;
    }

    public int getSoakingPower() {
        return soakingPower;
    }

    public int getOptimalRange() {
        return optimalRange;
    }

    public int getBalloons() {
        return balloons;
    }

}
