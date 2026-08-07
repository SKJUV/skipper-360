# Skipper 360 Environment Integration & Dynamic Whitelist Supervisor Script
# Sourced automatically by ~/.zshrc or ~/.bashrc

if [ -n "$ZSH_VERSION" ]; then
    # ZSH integration with fallback wrappers for Whitelist commands
    alias sudo='skipper run sudo'
    alias pacman='skipper run pacman'
    alias ssh='skipper run ssh'
    alias su='skipper run su'

elif [ -n "$BASH_VERSION" ]; then
    # BASH integration
    alias sudo='skipper run sudo'
    alias pacman='skipper run pacman'
    alias ssh='skipper run ssh'
    alias su='skipper run su'
fi
