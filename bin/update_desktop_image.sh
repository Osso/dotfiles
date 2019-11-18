#!/bin/sh

docker save pims | ssh desktop docker load
