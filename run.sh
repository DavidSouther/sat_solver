#!/bin/bash

FOLDER="./beam-planning/test_cases"

for TEST in $(ls $FOLDER) ; do 
    echo "Running $TEST"
    FILE="$FOLDER/$TEST"
    cat $FILE | cargo run 2>/dev/null | python "./beam-planning/evaluate.py" "$FILE"
done