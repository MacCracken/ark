#compdef ark

_ark() {
    local -a commands
    commands=(
        'install:Install one or more packages'
        'remove:Remove one or more packages'
        'search:Search for packages across all sources'
        'list:List installed packages'
        'info:Show detailed info about a package'
        'update:Check for updates across all sources'
        'upgrade:Upgrade packages with available updates'
        'status:Show ark version and system status'
        'hold:Hold packages to prevent upgrades'
        'unhold:Remove hold on packages'
        'verify:Verify integrity of installed packages'
        'history:Show transaction history'
    )

    _arguments -C \
        '--no-color[Disable colored output]' \
        '1:command:->cmds' \
        '*::arg:->args'

    case "$state" in
        cmds)
            _describe -t commands 'ark command' commands
            ;;
        args)
            case "$words[1]" in
                install)
                    _arguments \
                        '(-f --force)'{-f,--force}'[Force reinstall]' \
                        '(-g --group)'{-g,--group}'[Install group]:group:(desktop ai ml shell edge iot)' \
                        '*:package:'
                    ;;
                remove)
                    _arguments \
                        '--purge[Purge configuration files]' \
                        '*:package:'
                    ;;
                search)
                    _arguments \
                        '(-s --source)'{-s,--source}'[Filter by source]:source:(system marketplace flutter)' \
                        '*:query:'
                    ;;
                list|ls)
                    _arguments \
                        '--marketplace[Show marketplace only]' \
                        '--system[Show system only]' \
                        '--flutter[Show flutter only]'
                    ;;
                info|show) _arguments '1:package:' ;;
                hold) _arguments '*:package:' ;;
                unhold) _arguments '*:package:' ;;
                verify) _arguments '::package:' ;;
                history) _arguments '::count:' ;;
            esac
            ;;
    esac
}

_ark "$@"
