#!/bin/sh

BUILDER_IMAGE=${BUILDER_IMAGE:-'build-redstone-sequencer:dev'}
TARGET_DIR_VOLUME=${TARGET_DIR_VOLUME:-'ops-bedrock_rs_build'}
DOCKER=${DOCKER:-docker}


THIS_ZSH="$0:A"
THIS_BASH="$BASH_SOURCE"
THIS_DIR=$(cd "$(dirname ${BASH_SOURCE:-"$THIS_ZSH"})"; pwd)
PROJECTS_DIR=$(dirname $(dirname "$THIS_DIR"))

docker build -t "${BUILDER_IMAGE}" "${THIS_DIR}"

cd "${PROJECTS_DIR}/redstone-sequencer"



in-docker \
    --image "${BUILDER_IMAGE}" \
    --name build-redstone-sequencer \
    -v "${THIS_DIR}/cargo-git:/usr/local/cargo/git:rw" \
    -v "${THIS_DIR}/cargo-registry:/usr/local/cargo/registry:rw" \
    -v "${TARGET_DIR_VOLUME}:${PROJECTS_DIR}/redstone-sequencer/target:rw" \
    -v "${PROJECTS_DIR}/alloy:${PROJECTS_DIR}/alloy:ro" \
    -v "${PROJECTS_DIR}/reth:${PROJECTS_DIR}/reth:ro"
