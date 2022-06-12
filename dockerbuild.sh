#!/usr/bin/env bash

version=$1

sudo docker build -t ladefuchs-api:"$version" .
sudo docker save -o api.tar ladefuchs-api:"$version"
sudo chown -r "$USER" api.tar

# import docker load -i api.tar
