
###################
# history options #
###################

# don't put duplicate lines in the history. See bash(1) for more options
# ... or force ignoredups and ignorespace
HISTFILE="$HOME/.zsh/history"
HISTSIZE=10000
SAVEHIST=10000
# Ignore these commands
HISTIGNORE='ls:bg:fg:history:ll:exit:pwd:cd:cd -:exit:date:* --help'
# Share history between sessions
setopt SHARE_HISTORY
# Write history incrementally (after each command)
setopt INC_APPEND_HISTORY
# Don't record dupes in history
setopt HIST_IGNORE_ALL_DUPS
# Remove excess blanks in command line
setopt HIST_REDUCE_BLANKS
# Append to history (instead of overwriting it)
setopt APPEND_HISTORY
# Store datetime in history file
setopt EXTENDED_HISTORY


#########################
# included config files #
#########################

source ~/.zsh/prompt
source ~/.zsh/alias
source ~/.zsh/func
source ~/.zsh/zshmarks

# Homebrew virtualenvwrapper
source /usr/local/bin/virtualenvwrapper.sh

# Colorization of command line similar to fish shell
# source ~/src/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh


################
# autocomplete #
################

# Homebrew autocompletion
fpath=(/usr/local/share/zsh-completions $fpath)

# Custom autocompletion
fpath=(~/.zsh/autocomplete $fpath)

autoload -U compinit
compinit -u

# Add `killall` tab completion
zstyle ':completion:*:processes-names' command 'ps -e -o comm='

# Autocomplete projects for `w` and 'j' alias
compdef work_on_project=jump

zmodload zsh/complist
zstyle ':completion:*' menu select=2
bindkey -M menuselect ' ' accept-and-infer-next-history

# Example of completion of files in a directory
# #compdef c
# _files -W $PROJECTS -/

# Case-insensitive completion
zstyle ':completion:*' matcher-list 'm:{a-zA-Z}={A-Za-z}'


####################
# terminal options #
####################

# Use ^H to match the terminal and lxplus servers
# stty erase ^H
# Disable special handling of ^s and ^q by OS X Terminal
stty -ixon -ixoff


###############
# zsh options #
###############

# `**/qux` will enter `./foo/bar/baz/qux`
setopt autocd
# Case-insensitive globbing (used in pathname expansion)
unsetopt CASE_GLOB
# Complete on tab, leave expansion to _expand
bindkey '^I' complete-word
# Stop at directory separator when deleting words
autoload -U select-word-style
select-word-style bash
# Leave background programs running
setopt NO_HUP
# Don't beep when autocompleting
setopt NO_LIST_BEEP
# Use zmv, mass renaming module
autoload -U zmv
alias zmv="noglob zmv -W"
# Auto-correction in case of typos
setopt correctall
