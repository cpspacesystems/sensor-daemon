#!/usr/bin/env bash

gcc -o "$(basename $(pwd))" src/* -I include/
