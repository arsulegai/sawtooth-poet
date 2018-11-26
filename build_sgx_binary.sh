#!/bin/bash
WORK_DIR=/project/sawtooth-poet2/src
SGX_DIR=/tmp/sgxsdk

#setting SGX environment
source $SGX_DIR/environment

#Add proxy settings

echo "proxy = $http_proxy"
echo "SIM mode = $SGX_MODE"
echo "proxy type = manual" >> /etc/aesmd.conf
#echo "aesm proxy = http://proxy-iind.intel.com:911" >> /etc/aesmd.conf
echo "aesm proxy = $http_proxy" >> /etc/aesmd.conf
cat /etc/aesmd.conf

#starting aesm service
echo "start aesm service"
/opt/intel/libsgx-enclave-common/aesm/aesm_service &

#building SGX bridge and Enclave
cd $WORK_DIR
mkdir build
cd build
SGX_SIM_MODE=$SGX_MODE cmake $WORK_DIR/sgx
make 

export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:$WORK_DIR/build/bin
cd $WORK_DIR/core
