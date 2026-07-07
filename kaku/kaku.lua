-- ============================================================
--  kaku.lua — Config de référence (standalone)
--  Lancement : kaku --config-file ./kaku/kaku.lua start --cwd .
-- ============================================================

local wezterm = require("wezterm")
local config = wezterm.config_builder()

-- Fonts (fallback automatique si non installé)
config.font = wezterm.font_with_fallback({
  "JetBrainsMono Nerd Font",
  "FiraCode Nerd Font",
  "Menlo",
  "Monaco",
})

config.font_size = 14.0

-- Thème
config.color_scheme = "Catppuccin Mocha"

config.window_background_opacity = 0.95
config.macos_window_background_blur = 20

-- Keybindings utiles
config.keys = {
  { key = "d", mods = "CMD", action = wezterm.action.SplitHorizontal({}) },
  { key = "d", mods = "CMD|SHIFT", action = wezterm.action.SplitVertical({}) },
  { key = "w", mods = "CMD", action = wezterm.action.CloseCurrentPane({ confirm = true }) },
  { key = "LeftArrow", mods = "CMD|OPT", action = wezterm.action.ActivatePaneDirection("Left") },
  { key = "RightArrow", mods = "CMD|OPT", action = wezterm.action.ActivatePaneDirection("Right") },
  { key = "UpArrow", mods = "CMD|OPT", action = wezterm.action.ActivatePaneDirection("Up") },
  { key = "DownArrow", mods = "CMD|OPT", action = wezterm.action.ActivatePaneDirection("Down") },
}

return config
