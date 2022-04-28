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
# test case
RV32UI_P_TEST_BIN=`eval "find $RISCV_TESTS_BIN_PATH -type f -name \"rv32ui-p*\" ! -name \"*.dump\""`
RV32UM_P_TEST_BIN=`eval "find $RISCV_TESTS_BIN_PATH -type f -name \"rv32um-p*\" ! -name \"*.dump\""`
RV64UI_P_TEST_BIN=`eval "find $RISCV_TESTS_BIN_PATH -type f -name \"rv64ui-p*\" ! -name \"*.dump\""`
RV64UM_P_TEST_BIN=`eval "find $RISCV_TESTS_BIN_PATH -type f -name \"rv64um-p*\" ! -name \"*.dump\""`

unitTest() {
    printf "$INFO===RV32UI TESTS===$END\n"
    for file in $RV32UI_P_TEST_BIN; do
        val=`eval "basename $file"`
        printf "$INFO[%16s] $END" $val
        $PROGRAM --bin $file --xlen x32 --start-addr 0x1000
    done

    printf "$INFO===RV32UM TESTS===$END\n"
    for file in $RV32UM_P_TEST_BIN; do
        val=`eval "basename $file"`
        printf "$INFO[%16s] $END" $val
        $PROGRAM --bin $file --xlen x32 --start-addr 0x1000
    done

    printf "$INFO===RV64UI TESTS===$END\n"
    for file in $RV64UI_P_TEST_BIN; do
        val=`eval "basename $file"`
        printf "$INFO[%16s] $END" $val
        $PROGRAM --bin $file --xlen x64 --start-addr 0x1000
    done

    printf "$INFO===RV64UM TESTS===$END\n"
    for file in $RV64UM_P_TEST_BIN; do
        val=`eval "basename $file"`
        printf "$INFO[%16s] $END" $val
        $PROGRAM --bin $file --xlen x64 --start-addr 0x1000
    done
}


unitTest