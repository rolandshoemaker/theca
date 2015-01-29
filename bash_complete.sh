#  _   _                    
# | |_| |__   ___  ___ __ _ 
# | __| '_ \ / _ \/ __/ _` |
# | |_| | | |  __/ (_| (_| |
#  \__|_| |_|\___|\___\__,_|
#
# licensed under the MIT license <http://opensource.org/licenses/MIT>
#
# bash_complete.sh
#   (minimal) bash completion for the theca binary (also *seems* to work with zsh?)

_theca() {
	local commands
	local cur cmd

	COMPREPLY=()
	# cur=$(_get_cword "=")
	cmd="${COMP_WORDS[1]}"
	commands="add edit del clear transfer search info new-profile"
	global_opts="--profile --profile-folder --encrypted --yes --help --version"

	case "${cmd}" in
		add)
			COMPREPLY=( $(compgen -W \
        		"${global_opts} --started --urgent --none --body --editor -" -- "${cur}") )
        	return 0
			;;
		edit)
			COMPREPLY=( $(compgen -W \
        		"${global_opts} --started --urgent --body --editor -" -- "${cur}") )
        	return 0
			;;
		search)
			COMPREPLY=( $(compgen -W \
        		"${global_opts} --search-body --regex" -- "${cur}") )
        	return 0
			;;
		del|clear|transfer|new-profile|info)
			COMPREPLY=( $(compgen -W \
        		"${global_opts}" -- "${cur}") )
        	return 0
			;;
		help|version)
			return 0
			;;
	esac

	if [ ${COMP_CWORD} -eq 1 ]; then
        COMPREPLY=( $(compgen -W \
        	"${commands} --help --version" -- "${cur}") )
        return 0
    fi
} &&
complete -F _theca -o filenames theca
