#!/bin/sh

aclocal -I /usr/local/Cellar/gettext/0.19.2/share/aclocal && \
    automake -a && \
    autoconf && \
    ./configure
   # --with-openoffice-python=/Applications/LibreOffice.app/Contents/MacOS/python
   #--with-python=/opt/local/bin/python
