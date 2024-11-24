#!/bin/bash

CURR_DIR="$( cd "$( dirname $0)" && pwd )"
#echo CURR_DIR=$CURR_DIR

# PING="udp://39.105.43.146:5000?line=simon-home-ddns"
PING="udp://mylicense.rtcsdk.com:12002?line=simon-home-ddns"

export RUST_LOG=info

$CURR_DIR/aliyun-ddns --domain rtcsdk.com --domain 117890.xyz --rr simon.home --cli "$CURR_DIR/aliyun" --ping $PING
