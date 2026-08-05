#!/usr/bin/env bash
# ==============================================================================
# Skipper 360 — Script d'Installation
# Compile les binaires skipper et skipperd, puis les installe dans ~/.local/bin
# ==============================================================================

set -e

COLOR_CYAN="\033[1;36m"
COLOR_GREEN="\033[1;32m"
COLOR_YELLOW="\033[1;33m"
COLOR_RED="\033[1;31m"
COLOR_RESET="\033[0m"

echo -e "${COLOR_CYAN} Skipper 360 — Installation en cours...${COLOR_RESET}"
echo "--------------------------------------------------------"

# 1. Vérification des prérequis (Rust / Cargo)
if ! command -v cargo &> /dev/null; then
    echo -e "${COLOR_RED} Erreur : Rust/Cargo n'est pas installé sur votre système.${COLOR_RESET}"
    echo "Veuillez installer Rust via https://rustup.rs/ puis réessayez."
    exit 1
fi

INSTALL_DIR="${HOME}/.local/bin"
LIB_DIR="${HOME}/.local/lib"
mkdir -p "${INSTALL_DIR}" "${LIB_DIR}"

# 2. Compilation Release du Workspace
echo -e "${COLOR_YELLOW} Compilation des binaires et de la bibliothèque partagée (LD_PRELOAD)...${COLOR_RESET}"
cargo build --release

# 3. Copie des binaires et bibliothèques
echo -e "${COLOR_YELLOW} Installation des exécutables dans ${INSTALL_DIR} et ${LIB_DIR}...${COLOR_RESET}"
cp -f target/release/skipper "${INSTALL_DIR}/skipper"
cp -f target/release/skipperd "${INSTALL_DIR}/skipperd"
cp -f target/release/libskipper_preload.so "${LIB_DIR}/libskipper_preload.so"
chmod +x "${INSTALL_DIR}/skipper" "${INSTALL_DIR}/skipperd"

# 4. Configuration LD_PRELOAD globale transparente
echo -e "${COLOR_CYAN}⚙️ Configuration de l'interception transparente LD_PRELOAD...${COLOR_RESET}"
PROFILE_FILE="${HOME}/.bashrc"
if [[ -f "${HOME}/.zshrc" ]]; then
    PROFILE_FILE="${HOME}/.zshrc"
fi

if ! grep -q "libskipper_preload.so" "${PROFILE_FILE}"; then
    echo "" >> "${PROFILE_FILE}"
    echo "# Skipper 360 — Interception transparente globale" >> "${PROFILE_FILE}"
    echo "export LD_PRELOAD=\"${LIB_DIR}/libskipper_preload.so:\$LD_PRELOAD\"" >> "${PROFILE_FILE}"
    echo -e "${COLOR_GREEN} Configuration LD_PRELOAD ajoutée à ${PROFILE_FILE}${COLOR_RESET}"
fi

echo -e "${COLOR_GREEN}Installation terminée avec succès !${COLOR_RESET}"
echo "--------------------------------------------------------"
echo -e "Pour démarrer avec Skipper 360 :"
echo -e "  1. Exécutez : ${COLOR_CYAN}skipper init${COLOR_RESET}"
echo -e "  2. Lancez le diagnostic : ${COLOR_CYAN}skipper doctor${COLOR_RESET}"
echo -e "  3. Activez le daemon : ${COLOR_CYAN}skipper activate${COLOR_RESET}"
