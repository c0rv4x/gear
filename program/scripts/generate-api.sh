#!/bin/bash
#
# Generate and update gear-node api.

readonly ROOT_DIR="$(cd "$(dirname "$0")"/.. && pwd)"
readonly GEAR_NODE_DOCKER_IMAGE='ghcr.io/gear-tech/node:latest'
readonly GEAR_NODE_BIN="${ROOT_DIR}/res/gear-node"
readonly GENERATED_RS="${ROOT_DIR}/src/api/generated/metadata.rs"
readonly RPC_PORT='9933'
readonly SCRIPTS="${ROOT_DIR}/scripts"

###################
# Generated header
###################
function generate-header() {
    cat <<EOF
//! Auto generated by subxt-cli
//!
//! subxt codegen | rustfmt --edition=2021
//!
//! spec_version: $1
#![allow(clippy::all)]
#![allow(missing_docs)]
EOF
}

########################
# Usage of this script.
########################
function usage() {
    cat 1>&2 <<EOF
generate-api
Generate gear-node api.

USAGE:
    generate-api
EOF
}

###############################################################
# Check if the required binaries are installed in the machine.
###############################################################
function pre-check() {
    if ! [ -x "$(command -v subxt)" ]; then
        echo 'subxt not found, installing subxt...';

        if ! [ -x "$(command -v cargo)" ]; then
            echo 'cargo not found, installing rust...';
            curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal
        fi

        cargo install subxt-cli --version 0.22.0
    fi

    if ! [ -x "$(command -v rustfmt)" ]; then
        echo 'rustfmt not found, installing rustfmt...';
        rustup component add rustfmt
    fi
}


##########################################
# Run gear-node and capture spec-version.
##########################################
function spec-version() {
    spec_version=''

    # Pipe the stderr to read line into sub-shell
    ${GEAR_NODE_BIN} --tmp --dev --rpc-port ${RPC_PORT} 2>&1 > /dev/null |
        while [ "${spec_version}" == '' ] ; do
            read -r line
            if [[ "$line" == *"gear-"* ]]; then
                spec_version="$(echo ${line} | grep -Eo 'gear-[0-9]{3}' | sed 's/.*-//')"
                echo "$spec_version"
                break
            fi
        done &

    # TODO
    #
    # Optimize this double-while-loop if possible.
    while ! [ "${spec_version}" == '' ]; do
        kill -- -$!
    done
}

############################################
# Generate rust code for the gear-node api.
############################################
function main() {
    if [ "$#" -ne 0 ]; then
        usage
        exit 0
    fi

    # 0. Check if the required commands exist.
    pre-check

    # 1. Run gear-node and capture spec version
    spec_version="$(spec-version)"

    # 2. generate code
    sleep 5
    generate-header "${spec_version}" > "${GENERATED_RS}"
    subxt codegen --url "http://0.0.0.0:${RPC_PORT}" | rustfmt --edition=2021 >> "${GENERATED_RS}"

    echo "Updated gear-node api in ${GENERATED_RS}." >&2
    trap "kill 0" EXIT

    exit 0
}

main "$@"
