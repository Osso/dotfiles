#!/bin/sh

sshfs -o reconnect,ServerAliveInterval=15,ServerAliveCountMax=3 nas.localdomain:/mnt/array/home-assistant /mnt/home-assistant
