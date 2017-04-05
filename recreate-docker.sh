#!/bin/sh

docker-compose stop "$1" && docker-compose rm --force "$1" && docker-compose up -d --no-recreate "$1"
