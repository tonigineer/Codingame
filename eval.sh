#!/usr/bin/env bash
set -euo pipefail

num_games=${1:-10}

# Resolve game_dir relative to the script's location, not the caller's CWD
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
game_dir="$script_dir/bot-programming/trollfarm/assets/Troll-Farm"
pkg=trollfarm

cd "$game_dir"

wins=0
losses=0
draws=0
total_p1=0
total_p2=0

for ((i = 0; i < num_games; i++)); do
    # Random seed each game
    cur_seed=$RANDOM

    java -jar ./troll-farm-1.0-SNAPSHOT.jar \
        -p1 "./${pkg}" -p2 "./${pkg}-ref" -s -seed "$cur_seed"

    p1=$(jq '.scores."0"' /tmp/codingame/game.json)
    p2=$(jq '.scores."1"' /tmp/codingame/game.json)

    total_p1=$((total_p1 + p1))
    total_p2=$((total_p2 + p2))

    if   ((p1 > p2)); then result="WIN";  wins=$((wins + 1))
    elif ((p1 < p2)); then result="LOSS"; losses=$((losses + 1))
    else                   result="DRAW"; draws=$((draws + 1))
    fi

    printf "Game %3d (seed %5d): %3d vs %3d  [%s]\n" "$((i + 1))" "$cur_seed" "$p1" "$p2" "$result"
done

echo "────────────────────────────────────────"
echo "Games played: $num_games"
echo "Wins:   $wins"
echo "Losses: $losses"
echo "Draws:  $draws"
printf "Win rate: %.1f%%\n" "$(echo "scale=4; $wins / $num_games * 100" | bc)"
echo "Avg score: $((total_p1 / num_games)) vs $((total_p2 / num_games))"
