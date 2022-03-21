#!/bin/bash

# to print the color in terminal
INFO="\033[0;33m"
ERROR="\033[0;31m"
RIGHT="\033[0;32m"
END="\033[0m"

ROOT_PATH=$(dirname $(readlink -f "$0"))/../dependency

LIB_JSON_RPC_CPP_PATH=${ROOT_PATH}/libjson-rpc-cpp

configLibJsonRpcCpp() {
    sudo apt-get update
    sudo apt-get install libjsonrpccpp-dev libjsonrpccpp-tools
    cd ${ROOT_PATH}
    if [[ -d ${LIB_JSON_RPC_CPP_PATH} ]]; then
        echo -e "${RIGHT}libjson-rpc-cpp exist!${END}"
    else
        echo -e "${INFO}[no download]: git clone...${END}"
        git clone https://github.com/cinemast/libjson-rpc-cpp.git
    fi
}

configLibJsonRpcCpp