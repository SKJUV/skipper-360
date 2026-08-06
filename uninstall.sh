#!/usr/bin/env bash
set -e

BOLD="\033[1m"
GREEN="\033[32m"
RED="\033[31m"
YELLOW="\033[33m"
CYAN="\033[36m"
RESET="\033[0m"

echo -e "${BOLD}🛡️ Skipper 360 — Script de Désinstallation Completes${RESET}"
echo -e "${CYAN}──────────────────────────────────────────────────${RESET}"

# 1. Arrêt du daemon et du service systemd
echo -e "  [+] Arrêt des services et processus Skipper..."
if command -v systemctl &> /dev/null; then
    systemctl --user stop skipperd.service 2>/dev/null || true
    systemctl --user disable skipperd.service 2>/dev/null || true
    rm -f "$HOME/.config/systemd/user/skipperd.service"
    systemctl --user daemon-reload 2>/dev/null || true
    echo -e "  [OK] Service Systemd désactivé et supprimé."
fi

# Tuer tout processus skipperd résiduel si nécessaire
pkill -f skipperd 2>/dev/null || true

# 2. Suppression des binaires
echo -e "  [+] Suppression des binaires..."
BIN_DIR="$HOME/.cargo/bin"
rm -f "$BIN_DIR/skipper"
rm -f "$BIN_DIR/skipperd"
rm -f "$BIN_DIR/skipper-uninstall"
echo -e "  [OK] Binaires supprimés de ${GREEN}$BIN_DIR${RESET}"

# 3. Suppression des man pages
echo -e "  [+] Suppression des pages de manuel..."
MAN_DIR="$HOME/.local/share/man"
rm -f "$MAN_DIR/man1/skipper.1"
rm -f "$MAN_DIR/man8/skipperd.8"
echo -e "  [OK] Man pages supprimées."

# 4. Suppression des complétions shell
echo -e "  [+] Suppression des scripts d'autocomplétion shell..."
rm -f "$HOME/.zsh/completion/_skipper"
rm -f "$HOME/.local/share/bash-completion/completions/skipper"
rm -f "$HOME/.config/fish/completions/skipper.fish"
echo -e "  [OK] Complétions shell supprimées."

# 5. Nettoyage des fichiers rc (~/.zshrc, ~/.bashrc)
echo -e "  [+] Nettoyage de ~/.zshrc et ~/.bashrc..."
if [ -f "$HOME/.zshrc" ]; then
    sed -i '/# Skipper 360 Auto-Integration/d' "$HOME/.zshrc"
    sed -i '/fpath=(\$HOME\/\.zsh\/completion \$fpath)/d' "$HOME/.zshrc"
    sed -i '/skipper-env.sh/d' "$HOME/.zshrc"
    echo -e "  [OK] Références nettoyées dans ~/.zshrc"
fi

if [ -f "$HOME/.bashrc" ]; then
    sed -i '/# Skipper 360 Auto-Integration/d' "$HOME/.bashrc"
    sed -i '/skipper-env.sh/d' "$HOME/.bashrc"
    echo -e "  [OK] Références nettoyées dans ~/.bashrc"
fi

# 6. Suppression optionnelle ou complète des données de configuration et journaux
CONFIG_DIR="$HOME/.config/skipper360"
if [ -d "$CONFIG_DIR" ]; then
    echo -e "  [+] Suppression du répertoire de configuration (~/.config/skipper360)..."
    rm -rf "$CONFIG_DIR"
    echo -e "  [OK] Configuration et journaux nettoyés."
fi

echo -e "${CYAN}──────────────────────────────────────────────────${RESET}"
echo -e "${BOLD}${GREEN}[OK] Skipper 360 a été intégralement désinstallé de votre système.${RESET}"
