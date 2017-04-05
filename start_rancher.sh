#!/bin/sh

docker run -d --restart=always -v /var/rancher/cattle:/var/lib/cattle -v /var/rancher/mysql:/var/lib/mysql -v /var/rancher/log:/var/log/mysql -p 0.0.0.0:8080:8080 rancher/server