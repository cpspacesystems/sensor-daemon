#!/usr/bin/env bash

# Builds example project with prebuilt libsensor-daemon.a

gcc -o "$(basename $(pwd))" 'main.c' -I '../../include' -I '../../lib/include' -L '../../lib/' -L '../../build/export/' -l 'sensor-daemon' -l 'zenohpico'
