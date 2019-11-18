#!/bin/sh
set -e
set -x
reset
rsync -av --exclude=".git" --exclude="*.pyc" --delete . desktop:~/pims

if [ -z "$@"]; then
    entrypoint="bin/docker_entrypoint.sh bin/docker_tests.sh -rf -q -x --color=yes"
    volumes="-v '/home/osso/pims:/pims' -v 'persistent:/persistent'"
    docker_cmd="docker run --rm $volumes pims $entrypoint"
    # ssh -t desktop "cd pims && ls -d src/testspims/* | xargs -P 8 -n50 bin/run_tests -v"
    ssh -t desktop "cd pims && ls -d src/testspims/* | xargs -P7 -n34 $docker_cmd"
else
    ssh -t desktop "cd pims && bin/run_tests $@"
fi