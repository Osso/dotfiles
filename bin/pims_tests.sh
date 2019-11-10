#!/bin/sh
set -e
set -x
reset
rsync -av --exclude=.git --delete . desktop:~/pims
ssh desktop "cd pims && bin/run_tests $@"
