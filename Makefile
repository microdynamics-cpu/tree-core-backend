
JSON_RPC_PATH := dependency/libjson-rpc-cpp/src/examples

setup:
	chmod +x ./scripts/setup.sh
	./scripts/setup.sh

# need to check if the libjson is installed
rpc-test:
	jsonrpcstub $(JSON_RPC_PATH)/spec.json --cpp-server=AbstractStubServer --cpp-client=StubClient
	jsonrpcstub $(JSON_RPC_PATH)/spec.json --js-client=StubClient --js-client-file=$(JSON_RPC_PATH)/stubclient.js
	mkdir -p $(JSON_RPC_PATH)/gen
	mv abstractstubserver.h $(JSON_RPC_PATH)/gen
	mv stubclient.h $(JSON_RPC_PATH)/gen
	g++ $(JSON_RPC_PATH)/stubserver.cpp -ljsoncpp -lmicrohttpd -ljsonrpccpp-common -ljsonrpccpp-server -o $(JSON_RPC_PATH)/sampleserver
	g++ $(JSON_RPC_PATH)/stubclient.cpp -ljsoncpp -lcurl -ljsonrpccpp-common -ljsonrpccpp-client -o $(JSON_RPC_PATH)/sampleclient
	
.PHONY:
	setup rpc-test