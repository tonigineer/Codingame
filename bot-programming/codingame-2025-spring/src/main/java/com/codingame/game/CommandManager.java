package com.codingame.game;

import java.util.List;
import java.util.Optional;
import java.util.regex.Matcher;

import com.codingame.game.action.Action;
import com.codingame.game.action.ActionType;
import com.codingame.gameengine.core.GameManager;
import com.codingame.gameengine.core.MultiplayerGameManager;
import com.google.inject.Inject;
import com.google.inject.Singleton;

@Singleton
public class CommandManager {

    @Inject private MultiplayerGameManager<Player> gameManager;

    public void parseCommands(Player player, List<String> lines) {
        for (int actionIdx = 0; actionIdx < lines.size(); ++actionIdx) {
            String line = lines.get(actionIdx);
            try {
                Matcher match;
                String[] commands = line.split(";");
                Optional<Integer> agentIdOpt = getAgentId(commands);
                int startCommandsIdx = agentIdOpt.isPresent() ? 1 : 0;
                int agentId = agentIdOpt
                    .orElse(player.agents.get(actionIdx).getId());

                for (int i = startCommandsIdx; i < commands.length; ++i) {
                    String command = commands[i].trim();

                    boolean found = false;
                    try {
                        Agent agent = player.getAgentById(agentId);
                        if (agent == null) {
                            throw new InvalidInputException("a valid agent id", String.valueOf(agentId));
                        }
                        for (ActionType actionType : ActionType.values()) {
                            match = actionType.getPattern().matcher(command);
                            if (match.matches()) {
                                Action action = new Action(actionType);
                                actionType.getConsumer().accept(match, action);

                                // Agent can have 1 move, 1 message and 1 other (lets start by disallowing double move)
                                if (action.isMessage()) {
                                    agent.setMessage(action.getMessage().trim());
                                } else if (action.isMove() && (!Game.ALLOW_DOUBLE_MOVE || !agent.hasMoveAction())) {
                                    agent.setMoveAction(action);
                                } else {
                                    if (Game.ALLOW_DOUBLE_MOVE || !action.isMove()) {
                                        agent.setCombatAction(action);
                                    }
                                }
                                found = true;
                                break;
                            }
                        }
                        if (agent.getActions().size() >= 2 && agent.getMessage() != null) {
                            break;
                        }
                    } catch (Exception e) {
                        throw new InvalidInputException(Game.getExpected(command), command);
                    }

                    if (!found) {
                        throw new InvalidInputException(Game.getExpected(command), command);
                    }
                }

            } catch (InvalidInputException e) {
                deactivatePlayer(player, e.getMessage());
                gameManager.addToGameSummary(e.getMessage());
                gameManager.addToGameSummary(GameManager.formatErrorMessage(player.getNicknameToken() + ": disqualified!"));
            }
        }
    }

    private Optional<Integer> getAgentId(String[] tokens) {
        if (tokens.length > 1) {
            try {
                return Optional.of(Integer.parseInt(tokens[0]));
            } catch (NumberFormatException e) {
                return Optional.empty();
            }
        }
        return Optional.empty();
    }

    public void deactivatePlayer(Player player, String message) {
        player.deactivate(escapeHTMLEntities(message));
        player.setScore(-1);
    }

    private String escapeHTMLEntities(String message) {
        return message
            .replace("&lt;", "<")
            .replace("&gt;", ">");
    }
}
