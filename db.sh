#!/bin/sh

env SURREAL_EXPERIMENTAL_GRAPHQL=true surreal start --log strace --user root --pass root --bind 127.0.0.1:8999 surrealkv:rustblog.db
