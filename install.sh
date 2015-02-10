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

echo $"#  _   _                    "
echo $"# | |_| |__   ___  ___ __ _ "
echo $"# | __| '_ \ / _ \/ __/ _\` |"
echo $"# | |_| | | |  __/ (_| (_| |"
echo $"#  \__|_| |_|\___|\___\__,_|"
echo $"#"

if ! [[ -w "$INSTALL_PREFIX" ]]; then
	# if you don't have priv to write to INSTALL_PREFIX invoke 'sudo' before 'cp'
	PRIV_ESC="sudo"
fi

# copy all the stuff in FOLDERS_TO_INSTALL to INSTALL_PREFIX with parent directories
# yuh yuh
if [[ -e $INSTALL_PREFIX ]]; then
	echo $"# installing theca"
	for f in `find $FOLDERS_TO_INSTALL`; do
		if ! [[ -d "$f" ]]; then
			$PRIV_ESC cp --parents $f $INSTALL_PREFIX
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

echo $"#"
echo $"# installed `theca --version`"
echo $"#"

# first run type stuff
echo $"# would you like to setup the default profile folder and profile for theca?"
echo $"# this will create:"
echo $"#   $HOME/.theca"
echo $"#   $HOME/.theca/default.json"
echo $"# which can also be done with 'theca new-profile'"
select yn in "Yes" "No"; do
	case $yn in
		Yes)
			mkdir $HOME/.theca
			if [ "$?" -eq "0" ]; then
				echo $"# created $HOME/.theca"
			else
				echo $"# ERROR: couldn't create $HOME/.theca"
				exit 1
			fi
			theca new-profile
			if [ "$?" -eq "0" ]; then
				echo $"# created the default profile"
				echo $"#"
				echo $"# HAVE A FUN TIME"
			else
				echo $"# ERROR: couldn't create the default profile"
				exit 1
			fi
			break
		;;
		No)
			echo $"#"
			echo $"# ok bye!"
			exit
		;;
	esac
done
