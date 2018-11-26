#!/bin/bash
WORK_DIR=/project/sawtooth-poet/src
SGX_DIR=/tmp/sgxsdk

#setting SGX environment
source $SGX_DIR/environment

if [ $SGX_MODE == "FALSE" ]; then
    #Add proxy settings
    echo "proxy type = manual" >> /etc/aesmd.conf
    echo "aesm proxy = $http_proxy" >> /etc/aesmd.conf

    #starting aesm service
    echo "Starting aesm service"
    /opt/intel/libsgx-enclave-common/aesm/aesm_service &
fi

#building SGX bridge and Enclave
cd $WORK_DIR
mkdir build
cd build
SGX_SIM_MODE=$SGX_MODE cmake $WORK_DIR/sgx
make 

export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:$WORK_DIR/build/bin
cd $WORK_DIR/core
