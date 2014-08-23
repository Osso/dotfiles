
###########
# history #
###########

# don't put duplicate lines in the history. See bash(1) for more options
# ... or force ignoredups and ignorespace
HISTCONTROL=ignoredups:ignorespace:erasedups
# History file location
HISTFILE="$HOME/.bash/history"
# Max number of commmands in history
HISTSIZE=10000
# Max size of history file
HISTFILESIZE=1000000
# Write times in history
HISTTIMEFORMAT="%F %T "
# Do not write these commands in history
HISTIGNORE='ls:bg:fg:history:ll:exit:pwd:cd:cd -:exit:date:* --help'
# Write history to disk after each command
PROMPT_COMMAND="history -a; $PROMPT_COMMAND"
# append to the history file, don't overwrite it
shopt -s histappend


############
# includes #
############

source ~/.bash/prompt
source ~/.bash/alias
source ~/.bash/func

# MacPorts Installer addition on 2012-07-27_at_18:17:16: adding an appropriate PATH variable for use with MacPorts.
# export PATH=/opt/local/bin:/opt/local/sbin:$PATH
# Finished adapting your PATH environment variable for use with MacPorts.

# Homebrew virtualenvwrapper
source /usr/local/bin/virtualenvwrapper.sh


################
# autocomplete #
################

# Homebrew autocompletion
source /usr/local/etc/bash_completion
# Add tab completion for SSH hostnames based on ~/.ssh/config, ignoring wildcards
complete -o "default" -o "nospace" -W "$(grep "^Host" ~/.ssh/config | grep -v "[?*]" | cut -d " " -f2 | tr ' ' '\n')" scp sftp ssh;
# Add `killall` tab completion for common apps
complete -o "nospace" -W "Dock Finder SystemUIServer" killall;
# Enable tab completion for `g` by marking it as an alias for `git`
complete -o bashdefault -o default -o nospace -F _git g
# Enable tab completion for `v` by marking it as an alias for `vagrant`
complete -o bashdefault -o default -o nospace -F _vagrant v
# Autocomplete projects for `p` alias
complete -o nospace -W "$(/bin/ls ~/Projects)" p;


####################
# terminal options #
####################

# Use ^H to match the terminal and lxplus servers
# stty erase ^H
# Disable special handling of ^s and ^q by OS X Terminal
stty -ixon -ixoff


################
# bash options #
################

# Autocorrect typos in path names when using `cd`
shopt -s cdspell
# `**/qux` will enter `./foo/bar/baz/qux` (only avaible in homebrew bash)
# shopt -s autocd
# Case-insensitive globbing (used in pathname expansion)
shopt -s nocaseglob

