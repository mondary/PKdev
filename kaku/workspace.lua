-- workspace.lua — Config minimale (fonts + theme, pas de gui-startup)
-- Le layout des panes est créé par ./devpi via kaku cli split-pane
local wezterm = require("wezterm")
local config = wezterm.config_builder()

config.font = wezterm.font_with_fallback({
  "JetBrainsMono Nerd Font",
  "Menlo",
})
config.font_size = 13.0
config.color_scheme = "Catppuccin Mocha"
config.window_background_opacity = 0.95
config.macos_window_background_blur = 20

config.keys = {
  -- Basculer Dracula / Catppuccin
  {
    key = "T",
    mods = "CMD|SHIFT",
    action = wezterm.action_callback(function(win)
      local o = win:get_config_overrides() or {}
      o.color_scheme = (o.color_scheme == "Dracula") and "Catppuccin Mocha" or "Dracula"
      win:set_config_overrides(o)
    end),
  },
  -- Pane management
  { key = "d", mods = "CMD", action = wezterm.action.SplitHorizontal({}) },
  { key = "d", mods = "CMD|SHIFT", action = wezterm.action.SplitVertical({}) },
  { key = "w", mods = "CMD", action = wezterm.action.CloseCurrentPane({ confirm = true }) },
  -- Pane navigation
  { key = "LeftArrow", mods = "CMD|SHIFT", action = wezterm.action.ActivatePaneDirection("Left") },
  { key = "RightArrow", mods = "CMD|SHIFT", action = wezterm.action.ActivatePaneDirection("Right") },
  { key = "UpArrow", mods = "CMD|SHIFT", action = wezterm.action.ActivatePaneDirection("Up") },
  { key = "DownArrow", mods = "CMD|SHIFT", action = wezterm.action.ActivatePaneDirection("Down") },
  -- Tab navigation
  { key = "t", mods = "CMD", action = wezterm.action.SpawnTab("CurrentPaneDomain") },
  { key = "LeftArrow", mods = "CMD|OPT", action = wezterm.action.ActivateTabRelative(-1) },
  { key = "RightArrow", mods = "CMD|OPT", action = wezterm.action.ActivateTabRelative(1) },
}

return config
