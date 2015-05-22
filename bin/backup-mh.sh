ssh root@nas "rsync -avP root@mangahelpers.com:/var/backups /mnt/array/mh-backups/"
ssh root@nas "rsync -avP root@mangahelpers.com:/var/mangahelpers /mnt/array/mh-backups/"

