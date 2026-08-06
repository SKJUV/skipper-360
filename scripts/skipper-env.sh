# Skipper 360 Environment Integration Script
# This file is automatically sourced by your shell configuration (~/.zshrc, ~/.bashrc, etc.)

if [ -n "$ZSH_VERSION" ] || [ -n "$BASH_VERSION" ]; then
    # POSIX Shell Aliases for Skipper 360
    alias sudo='skipper run sudo'
    alias pacman='skipper run pacman'
    alias ssh='skipper run ssh'
    alias su='skipper run su'
fi
