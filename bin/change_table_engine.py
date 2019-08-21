#!/usr/bin/env python
engine = 'innodb'
host = 'localhost'
db_name = ''
user = ''
passwd = ''
skip_tables = ()

import MySQLdb

db = MySQLdb.connect(user=user, passwd=passwd, db=db_name, host=host)

c = db.cursor()
c.execute("show tables")

row = c.fetchone()
while row:
    table = row[0]
    print 'Converting Table: %s' % table
    e = db.cursor()
    e.execute("SHOW TABLE STATUS from `%s` LIKE '%s'" % (db_name, table))
    info = e.fetchone()
    if table in skip_tables or info[1] == engine:
        print 'Skipping'
        row = c.fetchone()
        continue
    e.execute('ALTER TABLE `%s` ENGINE = %s, tablespace ts_1 storage disk' % (MySQLdb.escape_string(table), engine))
    row = c.fetchone()
    print 'Done'
c.close()
