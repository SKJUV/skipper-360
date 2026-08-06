#!/usr/bin/env bash
set -e

BOLD="\033[1m"
GREEN="\033[32m"
BLUE="\033[34m"
YELLOW="\033[33m"
CYAN="\033[36m"
RESET="\033[0m"

echo -e "${BOLD}🛡️ Skipper 360 — Déploiement & Installation Automatisée${RESET}"
echo -e "${CYAN}──────────────────────────────────────────────────${RESET}"

ARCH=$(uname -m)
OS=$(uname -s)
echo -e "  [+] Architecture système : ${CYAN}${ARCH} (${OS})${RESET}"

# 1. Compilation des binaires release
echo -e "  [+] Compilation des binaires release (Cargo)..."
cargo build --release

BIN_DIR="$HOME/.cargo/bin"
CONFIG_DIR="$HOME/.config/skipper360"
mkdir -p "$BIN_DIR" "$CONFIG_DIR"

cp -f "target/release/skipper" "$BIN_DIR/skipper"
cp -f "target/release/skipperd" "$BIN_DIR/skipperd"
chmod +x "$BIN_DIR/skipper" "$BIN_DIR/skipperd"

echo -e "  [OK] Binaires installés dans ${GREEN}$BIN_DIR${RESET}"

# Copy environment integration script to persistent config folder
if [ -f "scripts/skipper-env.sh" ]; then
    cp -f "scripts/skipper-env.sh" "$CONFIG_DIR/skipper-env.sh"
fi

# 2. Installation et copie du script de désinstallation
if [ -f "uninstall.sh" ]; then
    cp -f "uninstall.sh" "$CONFIG_DIR/uninstall.sh"
    chmod +x "$CONFIG_DIR/uninstall.sh"
    # Also place a helper link or executable in BIN_DIR
    cp -f "uninstall.sh" "$BIN_DIR/skipper-uninstall"
    chmod +x "$BIN_DIR/skipper-uninstall"
fi

# 3. Installation du service Systemd Utilisateur (si présent)
if command -v systemctl &> /dev/null && [ -d "$HOME/.config" ]; then
    echo -e "  [+] Configuration du service Systemd (skipperd.service)..."
    SYSTEMD_USER_DIR="$HOME/.config/systemd/user"
    mkdir -p "$SYSTEMD_USER_DIR"
    cp -f "systemd/skipperd.service" "$SYSTEMD_USER_DIR/skipperd.service"
    
    systemctl --user daemon-reload || true
    systemctl --user enable --now skipperd.service || true
    echo -e "  [OK] Service Systemd utilisateur activé (skipperd.service)"
fi

# 4. Installation des Man-Pages
MAN_DIR="$HOME/.local/share/man"
mkdir -p "$MAN_DIR/man1" "$MAN_DIR/man8"
if [ -f "man/skipper.1" ]; then cp -f "man/skipper.1" "$MAN_DIR/man1/"; fi
if [ -f "man/skipperd.8" ]; then cp -f "man/skipperd.8" "$MAN_DIR/man8/"; fi
echo -e "  [OK] Pages de manuel installées dans ${GREEN}$MAN_DIR${RESET}"

# 5. Auto-configuration des Shells (ZSH, BASH, FISH)
echo -e "  [+] Configuration automatique des autocomplétions et du shell..."

# ZSH
if [ -f "$HOME/.zshrc" ]; then
    ZSH_COMP_DIR="$HOME/.zsh/completion"
    mkdir -p "$ZSH_COMP_DIR"
    "$BIN_DIR/skipper" completion zsh > "$ZSH_COMP_DIR/_skipper" 2>/dev/null || true

    if ! grep -q "skipper360/skipper-env.sh" "$HOME/.zshrc"; then
        echo "" >> "$HOME/.zshrc"
        echo "# Skipper 360 Auto-Integration" >> "$HOME/.zshrc"
        echo "fpath=(\$HOME/.zsh/completion \$fpath)" >> "$HOME/.zshrc"
        echo "[ -f \"$CONFIG_DIR/skipper-env.sh\" ] && source \"$CONFIG_DIR/skipper-env.sh\"" >> "$HOME/.zshrc"
        echo -e "  [OK] Intégration ajoutée à ${GREEN}~/.zshrc${RESET}"
    fi
fi

# BASH
if [ -f "$HOME/.bashrc" ]; then
    BASH_COMP_DIR="$HOME/.local/share/bash-completion/completions"
    mkdir -p "$BASH_COMP_DIR"
    "$BIN_DIR/skipper" completion bash > "$BASH_COMP_DIR/skipper" 2>/dev/null || true

    if ! grep -q "skipper360/skipper-env.sh" "$HOME/.bashrc"; then
        echo "" >> "$HOME/.bashrc"
        echo "# Skipper 360 Auto-Integration" >> "$HOME/.bashrc"
        echo "[ -f \"$CONFIG_DIR/skipper-env.sh\" ] && source \"$CONFIG_DIR/skipper-env.sh\"" >> "$HOME/.bashrc"
        echo -e "  [OK] Intégration ajoutée à ${GREEN}~/.bashrc${RESET}"
    fi
fi

# FISH
if [ -d "$HOME/.config/fish" ]; then
    FISH_COMP_DIR="$HOME/.config/fish/completions"
    mkdir -p "$FISH_COMP_DIR"
    "$BIN_DIR/skipper" completion fish > "$FISH_COMP_DIR/skipper.fish" 2>/dev/null || true
    echo -e "  [OK] Complétion Fish installée dans ${GREEN}$FISH_COMP_DIR/skipper.fish${RESET}"
fi

echo -e "${CYAN}──────────────────────────────────────────────────${RESET}"
echo -e "${BOLD}${GREEN}[OK] Skipper 360 a été déployé avec succès sur votre système ! 🎉${RESET}"
echo -e "  • Tapez ${YELLOW}skipper status${RESET} pour vérifier l'état du daemon."
echo -e "  • Pour désinstaller ultérieurement : exécuter ${YELLOW}skipper-uninstall${RESET} ou ${YELLOW}./uninstall.sh${RESET}."
echo -e "  • Pour appliquer la configuration shell immédiatement : ${YELLOW}source ~/.zshrc${RESET} ou relancez votre terminal."
