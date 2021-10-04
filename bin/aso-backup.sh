#!/bin/sh
set -e
set -x

dd if=/dev/nvme0n1 | gzip -1 - | ssh nas dd of=/mnt/array/aso.gz