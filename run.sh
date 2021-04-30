#!/bin/bash

FILE="$1"

cat $1 | cargo run 2>/dev/null 