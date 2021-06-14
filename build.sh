#!/bin/bash
# Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: MIT-0

set -eu

pushd lambda > /dev/null 2>&1

echo "build projects"

# build lambda project by using the container
# https://github.com/emk/rust-musl-builder
docker run -v "$(pwd)":/home/rust/src ekidd/rust-musl-builder cargo build --release 

# find the binaries we built
for dir in `find . -name "fitness-score-*"`; do
    # check the main.rs existance
    # if it exists, it has a binary we built
    main_rs_existance=`ls $dir/src | grep main.rs > /dev/null 2>&1; echo $?`

    if [ $main_rs_existance -eq 0 ]; then
        # replace all "-" to "_". (kebab case to snake case)
        binary_name=`echo $dir | tr - _ | tr -d ./`

        echo "assemble binary (".$binary_name.")"

        # copy the binary and rename it to "bootstrap"
        cp -f target/x86_64-unknown-linux-musl/release/$binary_name bootstrap

        # create the zip file
        zip $binary_name.zip bootstrap

        # remove the copied binary
        rm bootstrap
    fi
done

popd > /dev/null 2>&1
