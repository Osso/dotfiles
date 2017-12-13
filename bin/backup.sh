#!/bin/sh

if [ ! -b /dev/nbd0 ]; then
    echo "Missing backup device"
    exit 1
fi

sudo dd if=/dev/nvme0n1 of=/dev/nbd0 bs=1M status=progress conv=noerror,sync

