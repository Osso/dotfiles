#!/bin/env python
import subprocess  # For executing a shell command
import time
from datetime import datetime


def ping(host):
    """
    Returns True if host (str) responds to a ping request.
    Remember that a host may not respond to a ping (ICMP) request even if the host name is valid.
    """
    # Building the command. Ex: "ping -c 1 google.com"
    command = ['ping', '-c', '1', host]
    return subprocess.call(command, stdout=subprocess.DEVNULL) == 0


def reconnect():
    subprocess.call(['sudo', 'netplan', '--debug', 'apply'])


def main():
    while True:
        if ping('router.localdomain'):
            now = datetime.now().strftime("%m/%d %H:%M")
            print(f"{now} connected")
        else:
            reconnect()
        time.sleep(300)


if __name__ == '__main__':
    main()
