#!/usr/bin/env bash

gcc -o "$(basename $(pwd))" src/*.c src/drivers/*.c -I include/ -I lib/include/ -L lib/ -l zenohpico 
