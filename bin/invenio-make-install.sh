#!/bin/sh

CFG_INVENIO_SRCDIR=${CFG_INVENIO_SRCDIR:=~/src/invenio}
CFG_INSPIRE_SRCDIR=${CFG_INSPIRE_SRCDIR:=~/src/inspire}
CFG_INVENIO_PREFIX=${CFG_INVENIO_PREFIX:=/opt/invenio}

if [ -z "$CFG_INVENIO_PREFIX" ]; then
    echo "CFG_INVENIO_PREFIX is empty"
    exit 1
fi

# install sources:
echo "Installing invenio" && \
    cd "${CFG_INVENIO_SRCDIR}" && \
    make -s >/dev/null && \
    make -s install >/dev/null && \
    echo "Recreating configuration file" && \
    "${CFG_INVENIO_PREFIX}/bin/inveniocfg" --update-all >/dev/null && \
    echo "Done" || echo "Failed"
