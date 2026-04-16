# bash completion for ark
_ark() {
    local cur prev cmds opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    cmds="install remove search list info update upgrade status hold unhold verify history"

    case "$prev" in
        ark)
            COMPREPLY=($(compgen -W "$cmds --no-color" -- "$cur"))
            return ;;
        --no-color)
            COMPREPLY=($(compgen -W "$cmds" -- "$cur"))
            return ;;
        install)
            COMPREPLY=($(compgen -W "--force --group" -- "$cur"))
            return ;;
        remove|uninstall)
            COMPREPLY=($(compgen -W "--purge" -- "$cur"))
            return ;;
        search)
            COMPREPLY=($(compgen -W "--source" -- "$cur"))
            return ;;
        --source|-s)
            COMPREPLY=($(compgen -W "system marketplace flutter" -- "$cur"))
            return ;;
        --group|-g)
            COMPREPLY=($(compgen -W "desktop ai ml shell edge iot" -- "$cur"))
            return ;;
        list|ls)
            COMPREPLY=($(compgen -W "--marketplace --system --flutter" -- "$cur"))
            return ;;
    esac

    if [[ "$cur" == -* ]]; then
        COMPREPLY=($(compgen -W "--no-color --force --purge --source --group" -- "$cur"))
    fi
}
complete -F _ark ark
