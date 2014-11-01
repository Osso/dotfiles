ssh root@nas "rsync -avP root@mangahelpers.com:/home/ /mnt/array/mh-backups --exclude=/home/mysql --exclude=/home/lost+found"

