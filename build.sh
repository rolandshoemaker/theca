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
#   a simple build script because i forget things

INSTALL_DIR="/usr/local/bin"
MAN_DIR="/usr/local/share/man/man1"
BASH_COMPLETE_DIR="/usr/local/etc/bash_completion.d"
ZSH_COMPLETE_DIR="/usr/local/share/zsh/site-functions"

# check subcommand
case "$1" in
    # building the binary (just pass through to cargo)
    build)
        if command -v cargo >/dev/null 2>&1; then
            BUILD_CMD="cargo build"
            shift
            for arg in "$@"; do
                BUILD_CMD="$BUILD_CMD $arg"
            done
            eval "$BUILD_CMD"
            if [ "$?" -eq "0" ]; then
                if [[ "$@" =~ "--release" ]]; then
                    echo $"# built target/release/theca"
                else
                    echo $"# built target/theca"
                fi
            else
                if [[ "$@" =~ "--release" ]]; then
                    echo $"# couldn't build target/release/theca"
                    exit 1
                else
                    echo $"# couldn't build target/theca"
                    exit 1
                fi
            fi
        else
            echo $"cargo could not be found"
            exit 1
        fi
        ;;

    # build the man page, i can never remember the name of this thing
    build-man)
        if command -v md2man-roff >/dev/null 2>&1; then
            md2man-roff docs/THECA.1.md > docs/THECA.1
            echo $"# built THECA.1 man page"
        else
            echo $"# md2man-roff could not be found"
            exit 1
        fi
        ;;

    # install the binary in . so we don't have to bother about --dev/--release
    # binaries
    install)
        if [[ "$@" =~ "--release" ]]; then
            if [ -e "target/release/theca"]; then
                cp target/release/theca $INSTALL_DIR/
                echo $"# copied target/release/theca -> $INSTALL_DIR/theca"
            else
                echo $"# target/release/theca doesn't exist (did you run ./build.sh build --release)"
                exit 1
            fi
        else
            if [ -e "target/theca" ]; then
                cp target/theca $INSTALL_DIR/
                echo $"# copied target/theca -> $INSTALL_DIR/theca"
            else
                echo $"# target/theca doesn't exist (did you run ./build.sh build)"
                exit 1
            fi
        fi
        if [[ "$@" =~ "--bash-complete" ]]; then
            cp completion/bash_complete.sh $BASH_COMPLETE_DIR/theca
            echo $"# copied completion/bash_complete.sh -> $BASH_COMPLETE_DIR/theca"
        fi
        if [[ "$@" =~ "--zsh-complete" ]]; then
            cp completion/_theca $ZSH_COMPLETE_DIR/_theca
            echo $"# copied completion/_theca -> $ZSH_COMPLETE_DIR/theca"
        fi
        if [[ "$@" =~  "--man" ]]; then
            if [ -e "docs/THECA.1" ]; then
                cp docs/THECA.1 $MAN_DIR/
                echo $"# copied docs/THECA.1 -> $MAN_DIR/THECA.1"
            else
                echo $"# docs/THECA.1 doesn't exist"
                exit 1
            fi
        fi
        echo $"have fun :>"
        ;;

    # run all the tests in one place
    test)
        # run the rust tests
        if ! cargo test; then
            echo $"# rust tests did't pass!"
            exit 1
        fi

        # build the dev binary
        if ! cargo build; then
            echo $"# couldn't build the binary!"
            exit 1
        fi

        if [[ "$@" =~  "--travis" ]]; then
            python="python3.4"
        else
            python="python3"
        fi

        python_cmd="$python tests/theca_test_harness.py -tc"
        if [[ "$@" =~ "--release" ]]; then
            build_profile="release"
            python_cmd="$python_cmd target/release/theca"
        else
            build_profile="dev"
            python_cmd="$python_cmd target/theca"
        fi
        # run the python tests
        echo $"# running python harness tests"
        if ! eval "$python_cmd -pt"; then
            echo $"# [$build_profile] profile tests did not pass!"
            exit 1
        fi
        if ! eval "$python_cmd -jt"; then
            echo $"# [$build_profile] json output tests did not pass!"
            exit 1
        fi
        if ! eval "$python_cmd -tt"; then
            echo $"# [$build_profile] text output tests did not pass!"
            exit 1
        fi

        echo $"# it seems like everything is ok..."
        ;;

    # delete the target/ dir, the binary in . and the man page (if --man is used)
    clean)
        if [ -d "target" ]; then
            rm -r target
            echo $"# deleted ./target/"
        fi
        if [ -e "theca" ]; then
            rm theca
            echo $"# deleted ./theca"
        fi
        if [ -e "docs/THECA.1" ] && [[ "$@" =~  "--man" ]]; then
            rm docs/THECA.1
            echo $"# deleted ./docs/THECA.1"
        fi
        ;;

    # print the help
    *)
        echo $"Usage: $0 {build|build-man|test|install|clean}"
        exit 1
        ;;
esac
