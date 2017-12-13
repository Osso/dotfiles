#!/bin/sh
sudo sh -c 'echo "\nPackage: *\nPin: release a=trusty*\nPin-Priority: 1001\n\nPackage: *\nPin: origin download.01.org\nPin-Priority: -100\n" > /etc/apt/preferences.d/intel-removal'
sudo apt-get dist-upgrade
sudo rm /etc/apt/preferences.d/intel-removal
sudo rm /etc/apt/sources.list.d/intellinuxgraphics.list*
sudo apt-get update
echo "\n\n\n\n\n\n Remember to remove the i915-3.6-3.5-dkms and intel-linux-graphics-installer packages with \n\n sudo apt-get purge i915-3.6-3.5-dkms intel-linux-graphics-installer "

