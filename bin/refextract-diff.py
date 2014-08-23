#!/usr/bin/env python

import os
import sys
import difflib
import argparse

from xml.dom.minidom import parse
# from lxml.doctestcompare import LXMLOutputChecker
# from doctest import Example
from xml.parsers.expat import ExpatError

def read_xml(path):
    def valid_line(line):
        if line.lstrip(' \t').startswith('<subfield code="t">20'):
            return False
        if line.lstrip(' \t').startswith('<subfield code="v">'):
            return False
        return bool(line.strip())

    try:
        xml = parse(path)
    except ExpatError:
        with open(path) as f:
            xml = f.read()
    else:
        xml = xml.toprettyxml(encoding='utf-8').split('\n')
        xml = '\n'.join(line for line in xml if valid_line(line))
    return xml


def compare_results(results_dir, old_rev, new_rev, dest, verbose=0):
    for done, name in enumerate(sorted(os.listdir(os.path.join(results_dir, new_rev)))):
        if done % 100 == 99:
            print 'done', done + 1

        if verbose:
            print 'processing', name

        old_xml = read_xml(os.path.join(results_dir, old_rev, name))
        new_xml = read_xml(os.path.join(results_dir, new_rev, name))

        if old_xml != new_xml:
            msg = 'Difference in %s' % name
            print msg
            print >>dest, msg
            for line in difflib.unified_diff(old_xml.split('\n'), new_xml.split('\n')):
                print >>dest, line.strip('\n')
                dest.flush()

        # sys.stdout.writelines(diff)
        # checker = LXMLOutputChecker()
        # if not checker.check_output(old_xml, new_xml, 0):
        #     message = checker.output_difference(Example("", old_xml), new_xml, 0)
        #     print 'Difference in %s' % name
        #     print message.encode('utf-8')


def usage():
    print >>sys.stderr, "refextract-diff.py old_rev new_rev"
    sys.exit(1)


def main(options):
    old_rev, new_rev = options.old_rev, options.new_rev
    print 'Comparing %s to %s' % (old_rev, new_rev)

    filename = "%s_to_%s.diff" % (old_rev, new_rev)
    dest_path = os.path.join(os.getenv('AFSHOME'), filename)
    results_dir = os.path.join(os.getenv('AFSHOME'), 'refextract-results')
    with open(dest_path, 'w') as dest:
        compare_results(results_dir, old_rev, new_rev, dest, options.verbose)

    print 'Results written to', dest_path


def parse_args():
    """Parse CLI arguments"""
    parser = argparse.ArgumentParser(description='Refextract differ')
    parser.add_argument('--verbose', '-v', action='count')
    parser.add_argument('old_rev')
    parser.add_argument('new_rev')
    return parser.parse_args()



if __name__ == '__main__':
    try:
        main(parse_args())
    except KeyboardInterrupt:
        print >>sys.stderr, "Interrupted"
