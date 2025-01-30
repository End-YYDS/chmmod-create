#!/bin/bash
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
SCRIPT_PATH="$SCRIPT_DIR/$(basename "${BASH_SOURCE[0]}")"
PROGRAM_NAME="$(basename $SCRIPT_DIR)"
FRONTEND_DIR="$SCRIPT_DIR/src/frontend"

cd $SCRIPT_DIR
show_usage() {
    echo "Usage: $0 [command] [args...]"
    echo "Commands:"
    echo "  build                - Build the Rust project in release mode and copy libraries"
    echo "  run [args...]        - Run the Rust project with optional arguments"
    echo "  web-install          - Install frontend dependencies"
    echo "  web-dev [args...]    - Run yarn dev with optional arguments"
    echo "  web-build            - Run yarn build"
    echo "  help                 - Show this help message"
}

# 建置函數
build_release() {
    cd $SCRIPT_DIR
    cargo build --release
    mkdir -p "dist"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        LIB_EXT="dylib"
    elif [[ "$OSTYPE" == "linux"* ]]; then
        LIB_EXT="so"
    else
        echo "Unsupported OS: $OSTYPE" 
        exit 1
    fi
    find "target/release" -name "lib${PROGRAM_NAME}.*${LIB_EXT}" -exec cp {} "dist/" \;
}

get_extra_args() {
    shift
    echo "$@"
}

command="$1"
extra_args=$(get_extra_args "$@")

case "$command" in
    "build")
        build_release
        ;;
    "run")
        if [ -n "$extra_args" ]; then
            cargo run -- $extra_args
        else
            cargo run
        fi
        ;;
    "web-dev")
        cd $FRONTEND_DIR
        if [ -n "$extra_args" ]; then
            yarn dev $extra_args
        else
            yarn dev
        fi
        
        ;;
    "web-build")
        cd $FRONTEND_DIR
        yarn build
        ;;
    "web-install")
        cd $FRONTEND_DIR
        yarn install
        ;;
    "help" | "--help" | "-h")
        show_usage
        ;;
    "")
        echo "No command specified"
        show_usage
        exit 1
        ;;
    *)
        echo "Unknown command: $command"
        show_usage
        exit 1
        ;;
esac