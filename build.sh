#!/bin/bash
#  _   _                    
# | |_| |__   ___  ___ __ _ 
# | __| '_ \ / _ \/ __/ _` |
# | |_| | | |  __/ (_| (_| |
#  \__|_| |_|\___|\___\__,_|
#
# licensed under the MIT license <http://opensource.org/licenses/MIT>
#
# build.sh
#   a (linux) tool to build/install the theca binary and man page etc

# globals
INSTALL_DIR="/usr/bin"
MAN_DIR="/usr/local/share/man/man1"

case "$1" in
    build)
        if command -v cargo >/dev/null 2>&1; then
            BUILD_CMD="cargo build"
            shift
            for arg in "$@"; do
                BUILD_CMD="$BUILD_CMD $arg"
            done
            cargo update
            echo $"# dependencies udated"
            eval "$BUILD_CMD"
            if [ "$?" -eq "0" ]; then
                if [[ $@ =~ "--release" ]]; then
                    echo $"# built target/release/theca"
                    cp target/release/theca .
                    echo $"# copied target/release/theca to ."
                else
                    echo $"# built target/theca"
                    echo $"# copied target/theca to ."
                    cp target/theca .
                fi
            else
                echo $"couldn't build target/theca"
            fi
        else
            echo $"cargo could not be found"
            exit 1
            # there is probably a hard way to do this with
            # just rustc... but w/e for now
        fi
        ;;

    build-man)
        if command -v md2man-roff >/dev/null 2>&1; then
            md2man-roff docs/THECA.1.md > docs/THECA.1
            echo $"# built THECA.1 man page"
        else
            echo $"# md2man-roff could not be found"
            exit 1
        fi
        ;;

    install)
        if [ -e "theca" ]; then
            cp theca $INSTALL_DIR/
            echo $"# copied ./theca -> $INSTALL_DIR/theca"
        fi
        if [[ $@ =~  "--man" ]]; then
            if [ -e "docs/THECA.1" ]; then
                cp docs/THECA.1 $MAN_DIR/
                echo $"# copied docs/THECA.1 -> $MAN_DIR/THECA.1"
            fi
        fi
        ;;

    clean)
        if [ -d "target" ]; then
            rm -r target
            echo $"# deleted ./target/"
        fi
        if [ -e "theca" ]; then
            rm theca
            echo $"# deleted ./theca"
        fi
        if [ -e "docs/THECA.1" ]; then
            rm docs/THECA.1
            echo $"# deleted ./docs/THECA.1"
        fi
        ;;

    *)
        echo $"Usage: $0 {build|build-man|install|clean}"
        exit 1
        ;;
esac
