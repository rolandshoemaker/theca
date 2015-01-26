#!/bin/bash
#  _   _                    
# | |_| |__   ___  ___ __ _ 
# | __| '_ \ / _ \/ __/ _` |
# | |_| | | |  __/ (_| (_| |
#  \__|_| |_|\___|\___\__,_|
#
# license under the MIT license <http://opensource.org/licenses/MIT>
#
# build.sh
#   a (linux) tool to build/install the theca binary and man page etc

# globals
INSTALL_DIR="/usr/bin/"
MAN_DIR="/usr/local/share/man/man1"

case "$1" in
	build)
		if command -v cargo >/dev/null 2>&1; then
			BUILD_CMD="cargo build"
			shift
			for arg in "$@"; do
				BUILD_CMD="$BUILD_CMD $arg"
			done
			eval "$BUILD_CMD"
			if [ "$?" -eq "0" ]; then
				if [[ $@ =~ "--release" ]]; then
					echo $"built target/release/theca"
					echo $"copying target/release/theca to ."
					cp target/release/theca .
				else
					echo $"built target/theca"
					echo $"copying target/theca to ."
					cp target/theca .
				fi
			else
				echo $"couldn't build target/theca"
			fi
		else
			echo "cargo could not be found"
			# there is probably a hardway to do this with
			# just rustc... but w/e for now
		fi
		;;

	build-man)
		if command -v md2man-roff >/dev/null 2>&1; then
			md2man-roff docs/THECA.1.md > docs/THECA.1
			echo $"built THECA.1 man page"
		else
			echo $"md2man-roff could not be found"
		fi
		;;

	install)
		if [ -d "theca" ]; then
			echo $"copying ./theca -> $INSTALL_DIR/theca"
			cp theca $INSTALL_DIR/
		fi
		if [[ $@ =~  "--man" ]]; then
			if [ -d "docs/THECA.1" ]; then
				echo $"copying docs/THECA.1 -> $MAN_DIR/THECA.1"
				cp docs/THECA.1 $MAN_DIR/
			fi
		fi
		;;

	clean)
		if [ -d "target" ]; then
			rm -r target
		fi
		if [ -d "theca" ]; then
			rm theca
		fi
		if [ -d "docs/THECA.1" ]; then
			rm docs/THECA.1
		fi
		;;

	*)
		echo $"Usage: $0 {build|build-docs|install|clean}"
		exit 1

esac
