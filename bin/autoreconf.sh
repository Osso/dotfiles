#!/bin/sh

CFG_INVENIO_PREFIX=${CFG_INVENIO_PREFIX:=/opt/invenio}

aclocal && automake -a && autoconf && ./configure \
   --prefix="$CFG_INVENIO_PREFIX"  \
   --with-libintl-prefix=/opt/local \
   #--with-openoffice-python=/Applications/LibreOffice.app/Contents/MacOS/python
   #--with-python=/opt/local/bin/python
