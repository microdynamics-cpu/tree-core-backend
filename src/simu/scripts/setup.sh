#!/bin/bash

# to print the color in terminal
INFO="\033[0;33m"
ERROR="\033[0;31m"
RIGHT="\033[0;32m"
END="\033[0m"

ROOT_PATH=$(dirname $(readlink -f "$0"))/../dependency
RISCV_TESTS_PATH=${ROOT_PATH}/riscv-tests

configRiscvTests() {
    cd ${ROOT_PATH}
    if [[ -d ${RISCV_TESTS_PATH} ]]; then
        echo -e "${RIGHT}riscv tests exist!${END}"
    else
        echo -e "${INFO}[no download]: git clone...${END}"
        git clone https://github.com/riscv-software-src/riscv-tests.git
    fi
}

if [ ! -e $ROOT_PATH ]; then
    mkdir $ROOT_PATH
fi

configRiscvTests