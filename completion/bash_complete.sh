#  _   _                    
# | |_| |__   ___  ___ __ _ 
# | __| '_ \ / _ \/ __/ _` |
# | |_| | | |  __/ (_| (_| |
#  \__|_| |_|\___|\___\__,_|
#
# licensed under the MIT license <http://opensource.org/licenses/MIT>
#
# bash_complete.sh - v0.9.0
#   (minimal) bash completion for the theca binary

_theca() {
	local commands
	local cur cmd

	COMPREPLY=()
	cmd="${COMP_WORDS[1]}"
	cur="${COMP_WORDS[COMP_CWORD]}"
	commands="add edit del clear transfer transfer-from search info new-profile encrypt-profile decrypt-profile list-profiles --help --version"
	global_opts="--profile --profile-folder --encrypted --key"

	case "${cmd}" in
		add)
			COMPREPLY=( $(compgen -W \
        		"${global_opts} --started --urgent --body --editor - --yes" -- $cur) )
        	return 0
			;;
		edit)
			COMPREPLY=( $(compgen -W \
        		"${global_opts} --started --urgent --none --body --editor - --yes" -- $cur) )
        	return 0
			;;
		search)
			COMPREPLY=( $(compgen -W \
        		"${global_opts} --search-body --regex --limit --reverse --datesort --json --condensed" -- $cur) )
        	return 0
			;;
		del|clear|transfer|transfer-from|new-profile)
			COMPREPLY=( $(compgen -W \
        		"${global_opts} --yes" -- $cur) )
        	return 0
			;;
		encrypt-profile)
			COMPREPLY=( $(compgen -W \
				"${global_opts} --new-key"))
			return 0
			;;
		list-profiles)
			COMPREPLY=( $(compgen -W \
				"--profile-folder"))
			return 0
			;;
		decrypt-profile)
			COMPREPLY=( $(compgen -W \
				"${global_opts}"))
			return 0
			;;
		info)
			COMPREPLY=( $(compgen -W \
        		"${global_opts}" -- $cur) )
        	return 0
			;;
		help|version)
			return 0
			;;
	esac

	if [[ "${cmd}" =~ "^[0-9]+$" ]]; then
		COMPREPLY=( $(compgen -W \
    		"${global_opts} --json --condensed" -- $cur) )
    	return 0
	fi

	if [ ${COMP_CWORD} -eq 1 ]; then
        COMPREPLY=( $(compgen -W \
        	"${commands} --help --version --limit --reverse --datesort --json --condensed" -- $cur) )
        return 0
    fi
} &&
complete -F _theca -o filenames theca
