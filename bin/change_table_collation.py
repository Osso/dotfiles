collation = 'utf8_general_ci'
host = 'db'
db = ''
user = ''
passwd = ''
skip_tables = ()

import MySQLdb

db = MySQLdb.connect(user=user, passwd=passwd, db=db, host=host)

c = db.cursor()
c.execute("show tables")

row = c.fetchone()
while row:
    table = row[0]
    print 'Converting Table: %s' % table
    if table in skip_tables:
        print 'Skipping'
        row = c.fetchone()
        continue
    e = db.cursor()
    e.execute('ALTER TABLE `%s` COLLATE = %s' % (MySQLdb.escape_string(table), collation))
    row = c.fetchone()
    print 'Done'
c.close()
