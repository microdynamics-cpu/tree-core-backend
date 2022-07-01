default:
	@echo "-- VERILATE & BUILD --------"
	verilator -cc --exe --build -j top.v sim_main.cpp
	@obj_dir/Vtop

maintainer-copy::
clean mostlyclean distclean maintainer-clean::
	-rm -rf obj_dir *.log *.dmp *.vpd core
