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

INSTALL_PREFIX="/usr"
FOLDERS_TO_INSTALL="bin etc share"

if ! [[ -w "$INSTALL_PREFIX" ]]; then
	echo $"# ERROR: $INSTALL_PREFIX is not writable by $USER (perhaps you need to use sudo?)"
	exit 1
fi

if [[ -e $INSTALL_PREFIX ]]; then
	for f in `find $FOLDERS_TO_INSTALL`; do
		if ! [[ -d "$f" ]]; then
			cp --parents $f $INSTALL_PREFIX
			if [ "$?" -eq "0" ]; then
				echo $"# copied $f -> $INSTALL_PREFIX/$f"
			else
				echo $"# ERROR: couldn't copy $f -> $INSTALL_PREFIX/$f"
				exit 1
			fi
		fi
	done
else
	echo $"# ERROR: $INSTALL_PREFIX doesn't exist"
	exit 1
fi

echo $"# installed theca, yay!"
