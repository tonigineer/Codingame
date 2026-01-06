package com.codingame.game.action;

import java.util.function.BiConsumer;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import com.codingame.game.grid.Coord;

public enum ActionType {

    MOVE("^MOVE (?<x>\\d+) (?<y>\\d+)", (match, action) -> {
        action.setCoord(new Coord(Integer.valueOf(match.group("x")), Integer.valueOf(match.group("y"))));
    }),
    SHOOT("^SHOOT (?<id>\\d+)", (match, action) -> {
        action.setAgentId(Integer.valueOf(match.group("id")));
    }),
    THROW("^THROW (?<x>-?\\d+) (?<y>-?\\d+)", (match, action) -> {
        action.setCoord(new Coord(Integer.valueOf(match.group("x")), Integer.valueOf(match.group("y"))));
    }),
    HUNKER_DOWN("^(WAIT|HUNKER_DOWN)", ActionType::doNothing),
    MESSAGE("^MESSAGE (?<message>[^;]*)", (match, action) -> {
        action.setMessage(match.group("message"));
    });

    private Pattern pattern;
    private BiConsumer<Matcher, Action> consumer;

    private static void doNothing(Matcher m, Action a) {
    }

    ActionType(String pattern, BiConsumer<Matcher, Action> consumer) {
        this.pattern = Pattern.compile(pattern, Pattern.CASE_INSENSITIVE);
        this.consumer = consumer;
    }

    public Pattern getPattern() {
        return pattern;
    }

    public BiConsumer<Matcher, Action> getConsumer() {
        return consumer;
    }

}
