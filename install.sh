#!/usr/bin/env bash
set -e

BOLD="\033[1m"
GREEN="\033[32m"
BLUE="\033[34m"
YELLOW="\033[33m"
CYAN="\033[36m"
RESET="\033[0m"

echo -e "${BOLD}${BLUE}🛡️  Skipper 360 — Déploiement & Installation Automatisée${RESET}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"

ARCH=$(uname -m)
OS=$(uname -s)
echo -e "🖥️  ${BOLD}Architecture système détectée :${RESET} ${CYAN}${ARCH} (${OS})${RESET}"

# 1. Compilation des binaires release
echo -e "🔨 ${BOLD}Compilation optimisée des binaires release...${RESET}"
cargo build --release

BIN_DIR="$HOME/.cargo/bin"
mkdir -p "$BIN_DIR"

cp -f "target/release/skipper" "$BIN_DIR/skipper"
cp -f "target/release/skipperd" "$BIN_DIR/skipperd"
chmod +x "$BIN_DIR/skipper" "$BIN_DIR/skipperd"

echo -e "✅ Binaires installés avec succès dans ${GREEN}$BIN_DIR${RESET}"

# 2. Installation du service Systemd Utilisateur (si présent)
if command -v systemctl &> /dev/null && [ -d "$HOME/.config" ]; then
    echo -e "⚙️  ${BOLD}Configuration du service Systemd utilisateur (skipperd.service)...${RESET}"
    SYSTEMD_USER_DIR="$HOME/.config/systemd/user"
    mkdir -p "$SYSTEMD_USER_DIR"
    cp -f "systemd/skipperd.service" "$SYSTEMD_USER_DIR/skipperd.service"
    
    systemctl --user daemon-reload || true
    systemctl --user enable --now skipperd.service || true
    echo -e "✅ Service Systemd utilisateur activé (skipperd.service)"
fi

# 3. Installation des Man-Pages
MAN_DIR="$HOME/.local/share/man"
mkdir -p "$MAN_DIR/man1" "$MAN_DIR/man8"
cp -f "man/skipper.1" "$MAN_DIR/man1/"
cp -f "man/skipperd.8" "$MAN_DIR/man8/"
echo -e "📖 Pages de manuel installées dans ${GREEN}$MAN_DIR${RESET}"

# 4. Auto-configuration des Shells (ZSH, BASH, FISH)
echo -e "🐚 ${BOLD}Détection et configuration automatique des shells...${RESET}"

# ZSH
if [ -f "$HOME/.zshrc" ]; then
    COMP_DIR="$HOME/.zsh/completion"
    mkdir -p "$COMP_DIR"
    "$BIN_DIR/skipper" completion zsh > "$COMP_DIR/_skipper" 2>/dev/null || true

    if ! grep -q "skipper-env.sh" "$HOME/.zshrc"; then
        echo "" >> "$HOME/.zshrc"
        echo "# Skipper 360 Auto-Integration" >> "$HOME/.zshrc"
        echo "fpath=(\$HOME/.zsh/completion \$fpath)" >> "$HOME/.zshrc"
        echo "[ -f \"$(pwd)/scripts/skipper-env.sh\" ] && source \"$(pwd)/scripts/skipper-env.sh\"" >> "$HOME/.zshrc"
        echo "✅ Intégration ajoutée à ~/.zshrc"
    fi
fi

# BASH
if [ -f "$HOME/.bashrc" ]; then
    COMP_DIR="$HOME/.local/share/bash-completion/completions"
    mkdir -p "$COMP_DIR"
    "$BIN_DIR/skipper" completion bash > "$COMP_DIR/skipper" 2>/dev/null || true

    if ! grep -q "skipper-env.sh" "$HOME/.bashrc"; then
        echo "" >> "$HOME/.bashrc"
        echo "# Skipper 360 Auto-Integration" >> "$HOME/.bashrc"
        echo "[ -f \"$(pwd)/scripts/skipper-env.sh\" ] && source \"$(pwd)/scripts/skipper-env.sh\"" >> "$HOME/.bashrc"
        echo "✅ Intégration ajoutée à ~/.bashrc"
    fi
fi

# FISH
if [ -d "$HOME/.config/fish" ]; then
    FISH_COMP_DIR="$HOME/.config/fish/completions"
    mkdir -p "$FISH_COMP_DIR"
    "$BIN_DIR/skipper" completion fish > "$FISH_COMP_DIR/skipper.fish" 2>/dev/null || true
    echo "✅ Complétion Fish installée dans $FISH_COMP_DIR/skipper.fish"
fi

echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
echo -e "${BOLD}${GREEN}🎉 Skipper 360 a été déployé avec succès sur votre ordinateur !${RESET}"
echo -e "💡 Pour finaliser, faites : ${YELLOW}source ~/.zshrc${RESET} (ou relancez votre terminal)"
