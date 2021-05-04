#!/bin/bash

FOLDER="./beam-planning/test_cases"

for TEST in $(ls $FOLDER) ; do 
    echo "Running $TEST"
    FILE="$FOLDER/$TEST"
    cat $FILE | cargo run --features "elevation,analysis" | python "./beam-planning/evaluate.py" "$FILE"
done