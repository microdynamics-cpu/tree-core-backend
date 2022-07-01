SRC_FILE := $(shell find -name "*.v")

default:
	verilator -cc --exe --trace --assert -O3 --build --Mdir out --top-module rvcpu -j $(SRC_FILE) main.cpp
	@out/Vrvcpu

clean:
	rm -rf out
