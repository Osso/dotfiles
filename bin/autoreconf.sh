#!/bin/sh

CFG_INVENIO_PREFIX=${CFG_INVENIO_PREFIX:=/opt/invenio}

aclocal -I /usr/local/Cellar/gettext/0.19.2/share/aclocal && automake -a && autoconf && ./configure \
   --prefix="$CFG_INVENIO_PREFIX"  \
   # --with-libintl-prefix=/opt/macports \
   # --with-openoffice-python=/Applications/LibreOffice.app/Contents/MacOS/python
   #--with-python=/opt/local/bin/python
