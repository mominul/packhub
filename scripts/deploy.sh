#!/bin/bash

docker build --platform linux/amd64 -t packhub:latest -f images/server.Dockerfile .
docker tag packhub:latest mominul/packhub:latest
docker push mominul/packhub:latest
