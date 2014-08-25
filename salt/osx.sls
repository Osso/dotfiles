mypackages:
  pkg.installed:
    - pkgs:
      # Gnu utils like ls
      - coreutils
      # For Invenio
      - autoconf
      - automake
      # Version control
      - git
      - mercurial
      # Golang
      - go
      # Database
      - mariadb
      # Http client
      - wget
      - httpie
      # For watching network traffic
      - ngrep
      # Mass rename
      - rename
      # Required by zsh
      - gdbm
      # Required by shell prompt
      - vcprompt


# Shell completions
homebrew-completions:
  pkg.installed:
    - pkgs:
      - vagrant-completion
      - django-completion
      - pip-completion
      - zsh-completions
      - bash-completion
    - taps:
      - homebrew/completions


# File synching
my-packages:
  pkg.installed:
    - pkgs:
      - syncthing
    - taps:
      - osso/formula


# For installing the mac os x apps
brew-cask:
  pkg:
    - installed
    - taps:
      - caskroom/cask
