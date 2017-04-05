#!/bin/sh
set -e
set -x

ssh mh "/usr/bin/docker run -v /var/backups:/var/backups --net=host --rm --name dbbackup tools /root/db_backup.sh"
ssh root@nas "rsync -avP osso@mangahelpers.com:/var/backups /mnt/array/mh-backups/"
ssh root@nas "rsync -avP osso@mangahelpers.com:/var/mangahelpers /mnt/array/mh-backups/"
ssh mh 'sudo rm /var/backups/all-`date +%F`.sql.gz'