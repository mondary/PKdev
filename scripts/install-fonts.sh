#!/usr/bin/env bash
set -euo pipefail

echo "=== Installation des fonts pour le gestionnaire d'agents ==="

# Nerd Fonts — glyphs + icônes pour le TUI
NERD_FONTS=(
  "font-jetbrains-mono-nerd-font"
  "font-fira-code-nerd-font"
  "font-hack-nerd-font"
  "font-meslo-lg-nerd-font"
)

# Fonts proportionnelles — pour le branding / futurs composants GUI
DISPLAY_FONTS=(
  "font-inter"
  "font-poppins"
)

echo ""
echo "--- Nerd Fonts ---"
for f in "${NERD_FONTS[@]}"; do
  if brew list --cask "$f" &>/dev/null; then
    echo "  [ok] $f déjà installé"
  else
    echo "  [..] installation $f"
    brew install --cask "$f"
  fi
done

echo ""
echo "--- Fonts d'affichage ---"
for f in "${DISPLAY_FONTS[@]}"; do
  if brew list --cask "$f" &>/dev/null; then
    echo "  [ok] $f déjà installé"
  else
    echo "  [..] installation $f"
    brew install --cask "$f"
  fi
done

echo ""
echo "=========================================="
echo "  FUTURA — font commercial (non libre)"
echo "=========================================="
echo "  Futura n'est pas distribué via Homebrew."
echo "  Options :"
echo "    1. Installer manuellement depuis votre licence"
echo "    2. Alternative libre proche : 'font-jost' ou 'font-questrial'"
echo ""
echo "  Pour installer l'alternative Jost :"
echo "    brew install --cask font-jost"
echo "=========================================="
echo ""
echo "Terminé. Relancez votre terminal pour que les fonts soient prises en compte."
