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

# sh functions
p() {
	echo $"# $1"
}

err() {
	p "ERROR: $1"
	exit 1
}

ok() {
	if [ $? != 0 ]; then
		err "$1"
	fi
}

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
            ok "$BUILD_CMD failed"
            if [[ "$@" =~ "--release" ]]; then
                p "built target/release/theca"
            else
                p "built target/theca"
            fi
        else
            err "cargo could not be found"
        fi
        ;;

    # build the man page, i can never remember the name of this thing
    build-man)
        if command -v md2man-roff >/dev/null 2>&1; then
            md2man-roff docs/THECA.1.md > docs/THECA.1
            p "built THECA.1 man page"
        else
            err "md2man-roff could not be found"
        fi
        ;;

    # install the binary in . so we don't have to bother about --dev/--release
    # binaries
    install)
        if [[ "$@" =~ "--release" ]]; then
            if [ -e "target/release/theca"]; then
                cp target/release/theca $INSTALL_DIR/
                p "copied target/release/theca -> $INSTALL_DIR/theca"
            else
                err "target/release/theca doesn't exist (did you run ./build.sh build --release)"
            fi
        else
            if [ -e "target/theca" ]; then
                cp target/theca $INSTALL_DIR/
                p "copied target/theca -> $INSTALL_DIR/theca"
            else
                err "target/theca doesn't exist (did you run ./build.sh build)"
            fi
        fi
        if [[ "$@" =~ "--bash-complete" ]]; then
            cp completion/bash_complete.sh $BASH_COMPLETE_DIR/theca
            p "copied completion/bash_complete.sh -> $BASH_COMPLETE_DIR/theca"
        fi
        if [[ "$@" =~ "--zsh-complete" ]]; then
            cp completion/_theca $ZSH_COMPLETE_DIR/_theca
            p "copied completion/_theca -> $ZSH_COMPLETE_DIR/theca"
        fi
        if [[ "$@" =~  "--man" ]]; then
            if [ -e "docs/THECA.1" ]; then
                cp docs/THECA.1 $MAN_DIR/
                p "copied docs/THECA.1 -> $MAN_DIR/THECA.1"
            else
                err "docs/THECA.1 doesn't exist"
            fi
        fi
        p "have fun :>"
        ;;

    # run all the tests in one place
    test)
        # run the rust tests
        if ! cargo test; then
            err "rust tests did't pass!"
        fi

        # build the dev binary
        if ! cargo build; then
            err "couldn't build the binary!"
        fi

        if [[ "$@" =~  "--travis" ]]; then
            python="python3.4"
        else
            python="python3"
        fi

        python_cmd="$python tools/theca_test_harness.py --condensed -tc"
        if [[ "$@" =~ "--release" ]]; then
            build_profile="release"
            python_cmd="$python_cmd target/release/theca"
        else
            build_profile="dev"
            python_cmd="$python_cmd target/theca"
        fi
        # run the python tests
        p "running python harness tests"
        if ! eval "$python_cmd -pt"; then
            err "[$build_profile] profile tests did not pass!"
        fi
        if ! eval "$python_cmd -jt"; then
            err "[$build_profile] json output tests did not pass!"
        fi
        if ! eval "$python_cmd -tt"; then
            err "[$build_profile] text output tests did not pass!"
        fi

        p "it seems like everything is ok..."
        ;;

    # delete the target/ dir, the binary in . and the man page (if --man is used)
    clean)
        if [ -d "target" ]; then
            rm -r target
            p "deleted ./target/"
        fi
        if [ -e "theca" ]; then
            rm theca
            p "deleted ./theca"
        fi
        if [ -e "docs/THECA.1" ] && [[ "$@" =~  "--man" ]]; then
            rm docs/THECA.1
            p "deleted ./docs/THECA.1"
        fi
        ;;

    # print the help
    *)
        err "Usage: $0 {build|build-man|test|install|clean}"
        ;;
esac
