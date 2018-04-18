#!/bin/sh

apt list 'linux-image*' 2>/dev/null | grep 'installed' | cut -f1 -d'/' | grep $1 | xargs sudo apt remove -y
apt list 'linux-image*' 2>/dev/null | grep 'residual-config' | cut -f1 -d'/' | xargs sudo apt remove -y --purge
