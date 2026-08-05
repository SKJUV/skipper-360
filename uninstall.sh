#!/usr/bin/env bash
# ==============================================================================
# Skipper 360 — Script de Désinstallation
# Arrête le daemon, supprime les binaires installés et nettoie les configurations
# ==============================================================================

set -e

COLOR_CYAN="\033[1;36m"
COLOR_GREEN="\033[1;32m"
COLOR_YELLOW="\033[1;33m"
COLOR_RED="\033[1;31m"
COLOR_RESET="\033[0m"

echo -e "${COLOR_RED} Skipper 360 — Désinstallation en cours...${COLOR_RESET}"
echo "--------------------------------------------------------"

INSTALL_DIR="${HOME}/.local/bin"
CONFIG_DIR="${HOME}/.config/skipper360"

# 1. Arrêt du daemon s'il est actif
if command -v skipper &> /dev/null; then
    echo -e "${COLOR_YELLOW}🛑 Tentative de désactivation du daemon...${COLOR_RESET}"
    skipper deactivate 2>/dev/null || true
fi

# Tuer brutalement tout processus skipperd restant au cas où
pkill -f skipperd 2>/dev/null || true

# 2. Suppression des binaires
echo -e "${COLOR_YELLOW}Suppression des binaires dans ${INSTALL_DIR}...${COLOR_RESET}"
rm -f "${INSTALL_DIR}/skipper"
rm -f "${INSTALL_DIR}/skipperd"

# 3. Demande de confirmation pour supprimer le dossier de configuration
read -p "Voulez-vous également supprimer la configuration et les logs (~/.config/skipper360) ? (o/N) : " -n 1 -r
echo
if [[ $REPLY =~ ^[Oo]$ ]]; then
    echo -e "${COLOR_YELLOW} Suppression de ${CONFIG_DIR}...${COLOR_RESET}"
    rm -rf "${CONFIG_DIR}"
    echo -e "${COLOR_GREEN} Dossier de configuration supprimé.${COLOR_RESET}"
else
    echo -e "${COLOR_CYAN}Le dossier de configuration ${CONFIG_DIR} a été conservé.${COLOR_RESET}"
fi

echo -e "${COLOR_GREEN}Désinstallation de Skipper 360 terminée !${COLOR_RESET}"
