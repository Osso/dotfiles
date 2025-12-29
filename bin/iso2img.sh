#!/bin/sh

hdiutil convert -format UDRW -o "$1" "$2"