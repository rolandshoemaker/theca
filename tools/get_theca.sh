#!/bin/sh
#  _   _                    
# | |_| |__   ___  ___ __ _ 
# | __| '_ \ / _ \/ __/ _` |
# | |_| | | |  __/ (_| (_| |
#  \__|_| |_|\___|\___\__,_|
#
# licensed under the MIT license <http://opensource.org/licenses/MIT>
#
# get_theca.sh
#   super simple binary package downloader woot, won't
#   work until i setup bracewel.net but w/e for now...

p() {
	echo "get_theca: $1"
}

err() {
	echo "ERROR $1"
	exit 1
}

require() {
	if ! command -v $1 > /dev/null 2>&1; then
		err "need $1"
	fi
}

ok() {
	if [ $? != 0 ]; then
		err "$1"
	fi
}

delete() {
	if ! [ -f "$1" ]; then
		rm -Rf "$1"
		ok "couldn't delete $1"
	fi
}

get_host() {
	arch_uname=`uname -m`
	ok "couldn't use uname"
	if [ "$arch_uname" = "x86_64" ]; then
		arch="x86_64"
	elif [ "$arch_uname" = "i686" ]; then
		arch="i686"
	else
		err "binary install doesn't support $system_arch"
        fi
	system_uname=`uname -s`
	ok "couldn't use uname"
	if [ "$system_uname" = "Linux" ]; then
		system="unknown-linux-gnu"
	elif [ "$system_uname" = "Darwin" ]; then
		system="apple-darwin"
	else
		err "binary installer does not support $system_uname"
	fi
	echo "$arch-$system"
}

get_from_bracewel() {
	pkg_url="https://www.bracewel.net/theca/dist/theca-$1-$2.tar.gz"

	curl -O "$pkg_url"
	ok "couldn't download package from $pkg_url"

	tar zxvf "theca-$1-$2.tar.gz"
	ok "couldn't unpack theca-$1-$2.tar.gz"

	cd "theca-$1-$2"
	ok "couldn't enter package directory theca-$1-$2/"

	bash ./install.sh
	ok "couldn't execute the package installer"
}

uninstall_theca() {
	p "uninstalling theca!"
	usr_prefix="/usr/local"
	delete "$usr_prefix/bin/theca"
	delete "$usr_prefix/share/man/man1/theca.1"
	delete "$usr_prefix/share/zsh/site-functions/_theca"
	delete "$usr_prefix/etc/bash_completion.d/theca"
	p "byebye ._."
}

run() {
	require rm
	require mkdir
	require curl
	require tar
	require bash
	if [ "$#" != 0 ]; then
		for arg in "$@"; do
			case "$arg" in
				--uninstall)
					UNINSTALL=true
				;;
			esac
		done
		if [ ! -z "$UNINSTALL" ]; then
			uninstall_theca
		fi
	else
		release_channel="nightly"
		hosttriple=$( get_host )

		tmpdir=$(mktemp -d 2>/dev/null || mktemp -d -t 'theca-installer-tmp')
		startdir=`pwd`
		cd "$tmpdir"
		ok "failed to enter $tmpdir"

		get_from_bracewel "$release_channel" "$hosttriple"

		cd "$startdir"
		ok "failed to return to $startdir"
		delete "$tmpdir"
	fi
}

# so we don't accidently mess stuff up if download doesnt complete
run "$@"
