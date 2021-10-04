find /usr /bin /sbin /lib /lib64 -xdev \
    -path /usr/local -prune -false -o \
    -path /usr/share/mime -prune -false -o \
    -not -name '*.pyc' -a \
    -not -name '*.cache' -a \
    -type f | sort >/tmp/os.files
sort /var/lib/dpkg/info/*.list >/tmp/installed.files
comm -23 /tmp/os.files /tmp/installed.files