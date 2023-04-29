#!/bin/bash

version=$1

docker build -f Dockerfile -t timowuttke/krunch:${version} .
docker tag timowuttke/krunch:${version} timowuttke/krunch:latest
docker push -a timowuttke/krunch