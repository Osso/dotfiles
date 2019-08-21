#!/bin/sh
set -e
set -x

# ssh mh "/usr/bin/docker run -v /var/backups:/var/backups --net=host --rm --name dbbackup tools /root/db_backup.sh"
echo "Dumping DB"
ssh -t mh "~/db_backup.sh"
echo "Copying db backups to NAS"
ssh root@nas -t "rsync -avP osso@mangahelpers.com:~/backups /mnt/array/mh-backups/"
echo "Copying media backups to NAS"
ssh root@nas -t "rsync -avP osso@mangahelpers.com:/mnt/big_volume/mangahelpers /mnt/array/mh-backups/"
echo "Deleting old backups from MH server"
ssh mh -t 'sudo rm ~/backups/all-`date +%F`.sql.gz'