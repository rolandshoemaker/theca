#!/bin/bash
#  _   _                    
# | |_| |__   ___  ___ __ _ 
# | __| '_ \ / _ \/ __/ _` |
# | |_| | | |  __/ (_| (_| |
#  \__|_| |_|\___|\___\__,_|
#
# licensed under the MIT license <http://opensource.org/licenses/MIT>
#
# install.sh
#   simple bash script to install binaries, man page, bash+zsh completion
#   etc and run the first time stuff for the binary packages

INSTALL_PREFIX="/usr/local"
FOLDERS_TO_INSTALL="bin etc share"

p() {
	echo $"theca-installer: $1"
}

err() {
	p "ERROR: $1" >&2
	exit 1
}

ok() {
	if [ $? != 0 ]; then
		err "$1"
	fi
}

p "#  _   _                    "
p "# | |_| |__   ___  ___ __ _ "
p "# | __| '_ \ / _ \/ __/ _\` |"
p "# | |_| | | |  __/ (_| (_| |"
p "#  \__|_| |_|\___|\___\__,_|"
p "#"

if ! [[ -w "$INSTALL_PREFIX" ]]; then
	# if you don't have priv to write to INSTALL_PREFIX invoke 'sudo' before 'cp'
	PRIV_ESC="sudo"
fi

# copy all the stuff in FOLDERS_TO_INSTALL to INSTALL_PREFIX with parent directories
# yuh yuh
if [[ -e $INSTALL_PREFIX ]]; then
	p "# installing theca"
	for f in `find $FOLDERS_TO_INSTALL`; do
		if ! [[ -d "$f" ]]; then
			$PRIV_ESC cp --parents $f $INSTALL_PREFIX
			ok "couldn't copy $f -> $INSTALL_PREFIX/$f"
			p "# copied $f -> $INSTALL_PREFIX/$f"
		fi
	done
else
	err "$INSTALL_PREFIX doesn't exist"
fi

if ! command -v theca > /dev/null 2>&1; then
	err "can't run `theca` after install, not sure what's up with that"
fi

p "#"
p "# installed `theca --version`"
p "#"

# first run type stuff
p "# would you like to setup the default profile folder and profile for theca?"
p "# this will create:"
p "#   $HOME/.theca"
p "#   $HOME/.theca/default.json"
p "# which can also be done with 'theca new-profile'"
select yn in "Yes" "No"; do
	case $yn in
		Yes)
			p "#"
			mkdir $HOME/.theca
			ok "couldn't create $HOME/.theca"
			p "# created $HOME/.theca"
			theca new-profile
			ok "couldn't create default profile, this seems bad..."
			p "# created the default profile"
			p "#"
			p "# HAVE A FUN TIME"
			break
		;;
		No)
			p "#"
			p "# ok bye!"
			exit
		;;
	esac
done
