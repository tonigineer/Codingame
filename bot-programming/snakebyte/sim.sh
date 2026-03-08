#!/usr/bin/env bash

readonly EXECUTABLE="./target/release/snakebyte"

{
    for i in $(seq 1 50); do
        me=$((RANDOM % 2))
        opp=$((RANDOM % 2))
        echo "$me $opp"
    done
} | $EXECUTABLE
