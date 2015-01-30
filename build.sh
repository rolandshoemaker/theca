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
#   a (linux) tool to build/install the theca binary and man page and tab
#   completion stuff etc

# globals
INSTALL_DIR="/usr/local/bin"
MAN_DIR="/usr/local/share/man/man1"
BASH_COMPLETE_DIR="/usr/local/etc/bash_completion.d"
ZSH_COMPLETE_DIR="/usr/local/share/zsh/site-functions"

# check subcommand
case "$1" in
    # building the binary (just pass through to cargo then copy to .)
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
                    cp target/theca .
                    echo $"# copied target/theca to ."
                fi
            else
                echo $"couldn't build target/theca"
            fi
        else
            echo $"cargo could not be found"
            exit 1
            # there is a hard way to do this with just rustc but idk if
            # i want to write that shell script right now
        fi
        ;;

    # build the man page, i can never remember the name of this program
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
        if [ -e "theca" ]; then
            cp theca $INSTALL_DIR/
            echo $"# copied ./theca -> $INSTALL_DIR/theca"
            if [[ $@ =~ "--bash-complete" ]]; then
                cp completion/bash_complete.sh $BASH_COMPLETE_DIR/theca
                echo $"# copied ./bash_complete.sh -> $BASH_COMPLETE_DIR/theca"
            fi
            if [[ "$@" =~ "--zsh-complete" ]]; then
                cp completion/_theca $ZSH_COMPLETE_DIR/_theca
                echo $"# copied ./_theca -> $ZSH_COMPLETE_DIR/theca"
            fi
            if [[ $@ =~  "--man" ]]; then
                if [ -e "docs/THECA.1" ]; then
                    cp docs/THECA.1 $MAN_DIR/
                    echo $"# copied docs/THECA.1 -> $MAN_DIR/THECA.1"
                fi
            fi
            echo $"have fun :>"
        else
            echo $"# there is no theca binary in . did you forget to run './build.sh build'?"
            exit 1
        fi
        ;;

    # run all the tests in one place
    test)
        # run the rust tests
        if ! cargo test; then
            echo $"# rust tests did not pass!"
            exit 1
        fi

        build the binary
        if ! cargo build; then
            echo $"# couldn't build the binary!"
            exit 1
        fi

        if [[ $@ =~  "--travis" ]]; then
            python="python3.4"
        else
            python="python3"
        fi
        # should allow some way to set theca command by arg?
        python_cmd="$python tests/theca_test_harness.py -tc target/theca"
        # run the python tests
        echo $"# running python harness tests\n"
        if ! eval "$python_cmd -pt"; then
            echo $"# python harness profile tests did not pass!"
            exit 1
        fi
        echo $"\n"
        if ! eval "$python_cmd -jt"; then
            echo $"# python harness json output tests did not pass!"
            exit 1
        fi
        echo $"\n"
        if ! eval "$python_cmd -tt"; then
            echo $"# python harness text output tests did not pass!"
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
        if [ -e "docs/THECA.1" ] && [[ $@ =~  "--man" ]]; then
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
