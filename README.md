dotfiles
========

Provisioning of my os x

It is using ansible to install packages and configure the environment.
It will install [Homebrew](http://brew.sh) and install a bunch of python
packages via pip.


It covers the configuration of:
  * bash
  * zsh
  * git
  * mercurial
  * vim
  * nano
  * wget

There's a bunch of scripts in ~/bin


### Shells

Sets up:
  * prompt
  * history
  * aliases/funcs
  * autocompletion
  * env variables (including ls colors)
  * terminal options (enable ctrl-q/s)


### Sublime Text

This is my main text editor. The playbook will not install the plugins
for it yet. So they have to be installed manually.
Hopefully I get to add them to it soon.


Sets up:
  * keybindings
  * settings


### Vim

I rarely use VIM, so this configuration is mostly to make it a bit prettier
and easier to use.

Sets up:
  * [powerline](https://github.com/Lokaltog/powerline) status bar
  * disables a bunch of keys till I get familiar with the others
  * alt+direction to go around in insert mode
  * ctrl-a and ctrl-e work as expected
