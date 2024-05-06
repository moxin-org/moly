#!/bin/bash

WASMEDGE_CMD=wasmedge

if [ -n "$WASMEDGE_BUILD_DIR" ]; then
    WASMEDGE_CMD=$WASMEDGE_BUILD_DIR/tools/wasmedge/wasmedge
fi

check_wasmedge() {
    if command -v $WASMEDGE_CMD > /dev/null; then
        local wasmedge_output=$($WASMEDGE_CMD)
        if echo "$wasmedge_output" | grep -q 'nn-preload'; then
            return 0
        else
            echo "Wasmedge is installed but WASI NN plugin is not found."
            echo "Please download WASI NN plugin."
            echo "If you have already downloaded it, please set WASMEDGE_PLUGIN_PATH"
            return 1
        fi
    else
        echo "Please install wasmedge."
        echo "You can install wasmedge with the following command:"
        echo "curl -sSf https://raw.githubusercontent.com/WasmEdge/WasmEdge/master/utils/install_v2.sh | bash"
        return 1
    fi
}

main() {
    if [[ $# -eq 0 ]]; then
        echo "Usage: $0 [check|build|run] [--release]"
        exit 1
    fi

    if [[ $1 == "check" ]]; then
        check_wasmedge
        exit 0
    fi

    if ! check_wasmedge; then
        exit 1
    fi

    local release_mode=0
    if [[ $2 == "--release" ]]; then
        release_mode=1
    fi

    if [[ $1 == "build" ]]; then
        if [[ $release_mode -eq 1 ]]; then
            cargo build --release
        else
            cargo build
        fi
    elif [[ $1 == "run" ]]; then
        if [[ $release_mode -eq 1 ]]; then
            cargo run --release
        else
            cargo run
        fi
    else
        echo "Invalid argument. Use 'build', 'run', or 'check'."
        exit 1
    fi
}

main "$@"
