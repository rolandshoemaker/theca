#!/bin/sh
#  _   _                    
# | |_| |__   ___  ___ __ _ 
# | __| '_ \ / _ \/ __/ _` |
# | |_| | | |  __/ (_| (_| |
#  \__|_| |_|\___|\___\__,_|
#
# licensed under the MIT license <http://opensource.org/licenses/MIT>
#
# get_theca.sh - v0.9.1
#   super simple binary package downloader that invokes
#   the install.sh installer script, pretty much ready
#   for action now

p() {
	echo "get_theca: $1"
}

err() {
	p "ERROR $1" >&2
	exit 1
}

has() {
	if command -v $1 > /dev/null 2>&1; then
		return 0
	else
		return 1
	fi
}

require() {
	if ! has $1; then
		err "$1 is required for this script!"
	fi
}

ok() {
	if [ $? != 0 ]; then
		err "$1"
	fi
}

delete() {
	if ! [ -f "$1" ]; then
		rm -rf "$1"
		ok "couldn't delete $1"
	fi
}

get_yn() {
	local prompt
	local resp
	local default
	local question="$1"
	if [ "$#" -eq "2" ]; then
		if [ ! -z "$2" ]; then
			prompt="Y/n"
			default=0
		else
			prompt="y/N"
			default=1
		fi
	else
		prompt="y/n"
	fi
	while true; do
	    read -p "$question [$prompt]: " yn
	    case $yn in
	        [yY]*) resp=0; break;;
	        [nN]*) resp=1; break;;
			"")
				if [ "$#" -eq "2" ]; then
					resp=$default; break
				else
					p "Please enter y or n."
				fi
			;;
	        *) p "Please enter y or n.";;
	    esac
	done
	return $resp
}

get_host() {
	local arch_uname=`uname -m`
	ok "couldn't use uname"
	if [ "$arch_uname" = "x86_64" ]; then
		local arch="x86_64"
	elif [ "$arch_uname" = "i686" ]; then
		local arch="i686"
	else
		err "binary install doesn't support $system_arch"
        fi
	local system_uname=`uname -s`
	ok "couldn't use uname"
	if [ "$system_uname" = "Linux" ]; then
		local system="unknown-linux-gnu"
	elif [ "$system_uname" = "Darwin" ]; then
		local system="apple-darwin"
	else
		err "binary installer does not support $system_uname"
	fi
	echo "$arch-$system"
}

get_from_bracewel() {
	local pkg_url="https://static.bracewel.net/theca/dist/theca-$1-$2.tar.gz"

	curl -O "$pkg_url"
	ok "couldn't download package from $pkg_url"

	tar zxvf "theca-$1-$2.tar.gz"
	ok "couldn't unpack theca-$1-$2.tar.gz"

	cd "theca-$1-$2"
	ok "couldn't enter package directory theca-$1-$2/"

	if has bash; then
		bash ./install.sh <&1
	fi
	ok "package installer exited badly"
}

uninstall_theca() {
	if get_yn "are you sure you want to uninstall?"; then
		p "uninstalling theca!"
		delete "$1/bin/theca"
		delete "$1/share/man/man1/theca.1"
		delete "$1/share/zsh/site-functions/_theca"
		delete "$1/etc/bash_completion.d/theca"
		p "byebye ._."
	fi
}

run() {
	require rm
	require mkdir
	require curl
	require tar

	local INSTALL_PREFIX="/usr/local"
	local release_channel="nightly"

	for arg in "$@"; do
		case "$arg" in
			--uninstall)
				UNINSTALL=true
			;;
			--nightly)
				release_channel="nightly"
			;;
		esac
	done
	if [ ! -z "$UNINSTALL" ]; then
		uninstall_theca INSTALL_PREFIX
	else
		local hosttriple=$( get_host )
		local tmpdir=$(mktemp -d 2>/dev/null || mktemp -d -t 'theca-installer-tmp')

		local startdir=`pwd`
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
