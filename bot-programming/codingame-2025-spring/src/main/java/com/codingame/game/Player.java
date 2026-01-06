package com.codingame.game;

import java.util.ArrayList;
import java.util.List;

import com.codingame.gameengine.core.AbstractMultiplayerPlayer;
import com.google.common.base.Objects;

public class Player extends AbstractMultiplayerPlayer {

    List<Agent> agents;
    int points;

    public Player() {
        agents = new ArrayList<>(5);
    }

    public void init() {
    }

    @Override
    public int getExpectedOutputLines() {
        return getLiveAgentCount();
    }

    public int getLiveAgentCount() {
        return (int) agents.stream().filter(a -> !a.dying).count();
    }

    public void reset() {
        agents.stream().forEach(Agent::reset);
    }

    public Agent getAgent(int idx) {
        return agents.get(idx);
    }

    public void addScore(int points) {
        this.points += points;
        setScore(this.points);
    }

    public Agent getAgentById(Integer agentId) {
        for (Agent agent : agents) {
            if (Objects.equal(agent.getId(), agentId)) {
                return agent;
            }
        }
        return null;
    }

}
