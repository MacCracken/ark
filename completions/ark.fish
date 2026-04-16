# fish completion for ark

set -l commands install remove search list info update upgrade status hold unhold verify history

complete -c ark -n "not __fish_seen_subcommand_from $commands" -l no-color -d "Disable colored output"
complete -c ark -n "not __fish_seen_subcommand_from $commands" -a "$commands"

# install
complete -c ark -n "__fish_seen_subcommand_from install" -s f -l force -d "Force reinstall"
complete -c ark -n "__fish_seen_subcommand_from install" -s g -l group -d "Install group" -xa "desktop ai ml shell edge iot"

# remove
complete -c ark -n "__fish_seen_subcommand_from remove" -l purge -d "Purge configuration files"

# search
complete -c ark -n "__fish_seen_subcommand_from search" -s s -l source -d "Filter by source" -xa "system marketplace flutter"

# list
complete -c ark -n "__fish_seen_subcommand_from list" -l marketplace -d "Show marketplace only"
complete -c ark -n "__fish_seen_subcommand_from list" -l system -d "Show system only"
complete -c ark -n "__fish_seen_subcommand_from list" -l flutter -d "Show flutter only"
