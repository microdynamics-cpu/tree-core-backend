#!/bin/bash

# to print the color in terminal
INFO="\033[0;33m"
ERROR="\033[0;31m"
RIGHT="\033[0;32m"
END="\033[0m"

ROOT_PATH=$(dirname $(readlink -f "$0"))/../dependency
RISCV_TESTS_PATH=${ROOT_PATH}/riscv-tests
RISCV_TESTS_BIN_PATH=${RISCV_TESTS_PATH}/build/share/riscv-tests/isa
PROGRAM=$(dirname $(readlink -f "$0"))/../target/debug/treecore_simu


unitTest() {
    BIN_FILES=`eval "find $RISCV_TESTS_BIN_PATH -type f ! -name \"rv32um-*\" ! -name \"rv32ui-v-*\" ! -name \"*.dump\" ! -name \"Makefile\" ! -name \".gitignore\""`
    for file in $BIN_FILES; do
        # echo $file
        $PROGRAM $file
    done
    # echo $BIN_FILES
}
    

unitTest