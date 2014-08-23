#!/bin/sh

CFG_INVENIO_SRCDIR=${CFG_INVENIO_SRCDIR:=~/src/invenio}
CFG_INSPIRE_SRCDIR=${CFG_INSPIRE_SRCDIR:=~/src/inspire}
CFG_INVENIO_PREFIX=${CFG_INVENIO_PREFIX:=/opt/invenio}

if [ -z "$CFG_INVENIO_PREFIX" ]; then
    echo "CFG_INVENIO_PREFIX is empty"
    exit 1
fi


echo "Deleting invenio" && \
    rm -rf "$CFG_INVENIO_PREFIX/lib/invenio" &&
    rm -rf "$CFG_INVENIO_PREFIX/var" &&
    rm -rf "$CFG_INVENIO_PREFIX/etc" &&
    echo "Installing invenio" && \
    cd "${CFG_INVENIO_SRCDIR}" && \
    make -s >/dev/null && \
    make -s install >/dev/null && \
    #cd po && make install-data-yes && cd .. && \
    echo "Recreating configuration file" && \
    cp "$CFG_INVENIO_SRCDIR/../invenio-local.conf" "$CFG_INVENIO_PREFIX/etc/" &&
    "${CFG_INVENIO_PREFIX}/bin/inveniocfg" --update-all >/dev/null && \
    inveniocfg --drop-tables \
               --create-tables \
               --create-demo-site \
               --load-demo-records \
               --yes-i-know && \
    echo "Done" || echo "Failed"


mkdir -p /opt/invenio/var/tmp/ooffice-tmp-files