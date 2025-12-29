local wezterm = require 'wezterm'
local act = wezterm.action
local config = wezterm.config_builder()
local home = os.getenv('HOME') or '/home/user'

-- Native Wayland (requires niri window rule for wezterm)
config.enable_wayland = true

-- Match monitor refresh rate (85Hz)
config.max_fps = 85
config.animation_fps = 85

-- Font
config.font = wezterm.font('MesloLGS NF', { weight = 'Medium' })
config.font_size = 14.0

-- Appearance
config.window_background_opacity = 0.9

-- Match kitty color rendering
config.bold_brightens_ansi_colors = true
config.front_end = 'WebGpu'
config.window_decorations = 'NONE'
config.enable_tab_bar = true
config.hide_tab_bar_if_only_one_tab = false
config.use_fancy_tab_bar = false
config.tab_bar_at_bottom = true
config.tab_max_width = 300
config.show_tab_index_in_tab_bar = false
config.show_new_tab_button_in_tab_bar = false

-- Tab titles: SSH → title, shell → directory, interpreter → title, other → process
wezterm.on('format-tab-title', function(tab, tabs, panes, cfg, hover, max_width)
    local pane = tab.active_pane
    local process = pane.foreground_process_name:match('([^/]+)$') or ''
    local shells = { zsh = true, bash = true, brush = true, fish = true, sh = true }
    local interpreters = { node = true, python = true, python3 = true, ruby = true, perl = true }
    local title

    if process == 'ssh' or interpreters[process] then
        -- SSH or interpreter: use pane title (set by remote shell or app)
        title = pane.title
    elseif shells[process] or process == '' then
        -- Shell: show directory name
        local cwd = pane.current_working_dir
        if cwd then
            local path = cwd.file_path or ''
            if path == home then
                title = '~'
            else
                title = path:gsub('^' .. home .. '/', '~/'):match('([^/]+)$') or path
            end
        else
            title = '~'
        end
    else
        -- Other programs: use process name
        title = process
    end

    local bg = tab.is_active and '#ca9ee6' or '#3a3a3a'
    local fg = tab.is_active and '#1e1e1e' or '#909090'

    if tab.tab_index == 0 then
        return {
            { Background = { Color = bg } },
            { Foreground = { Color = fg } },
            { Text = ' ' .. title .. ' ' },
        }
    else
        return {
            { Background = { Color = '#1e1e1e' } },
            { Foreground = { Color = '#888888' } },
            { Text = '│' },
            { Background = { Color = bg } },
            { Foreground = { Color = fg } },
            { Text = ' ' .. title .. ' ' },
        }
    end
end)

-- Catppuccin Frappe colors (customized to match kitty)
config.colors = {
    foreground = '#d4d4d4',
    background = '#262626',
    cursor_bg = '#f2d5cf',
    cursor_fg = '#303446',
    cursor_border = '#f2d5cf',
    selection_fg = '#c6d0f5',
    selection_bg = '#626880',
    scrollbar_thumb = '#626880',
    split = '#737994',
    ansi = {
        '#51576d', -- black
        '#e78284', -- red
        '#a6d189', -- green
        '#e5c890', -- yellow
        '#6c8ed4', -- blue
        '#f4b8e4', -- magenta
        '#81c8be', -- cyan
        '#b5bfe2', -- white
    },
    brights = {
        '#626880', -- bright black
        '#e78284', -- bright red
        '#a6d189', -- bright green
        '#e5c890', -- bright yellow
        '#6c8ed4', -- bright blue
        '#f4b8e4', -- bright magenta
        '#81c8be', -- bright cyan
        '#a5adce', -- bright white
    },
    -- Extended palette colors (to match kitty)
    indexed = {
        [33] = '#6c8ed4', -- 256-color blue used by dircolors
    },
    tab_bar = {
        background = '#1e1e1e',
        active_tab = {
            bg_color = '#ca9ee6',
            fg_color = '#1e1e1e',
        },
        inactive_tab = {
            bg_color = '#3a3a3a',
            fg_color = '#909090',
        },
        inactive_tab_hover = {
            bg_color = '#414559',
            fg_color = '#c6d0f5',
        },
        new_tab = {
            bg_color = '#1e1e1e',
            fg_color = '#909090',
        },
        new_tab_hover = {
            bg_color = '#3a3a3a',
            fg_color = '#c6d0f5',
        },
    },
}

-- Bell
config.audible_bell = 'Disabled'

-- No close confirmation
config.window_close_confirmation = 'NeverPrompt'

-- Shell
config.default_prog = { 'bash', '--login' }

-- Mouse: hide immediately when typing
config.hide_mouse_cursor_when_typing = false

-- Scrollback
config.scrollback_lines = 10000

-- Key bindings
config.keys = {
    -- Copy/Paste (matching kitty: Ctrl+C copy, Ctrl+Shift+C for SIGINT)
    { key = 'c', mods = 'CTRL',       action = act.CopyTo 'Clipboard' },
    { key = 'c', mods = 'CTRL|SHIFT', action = act.SendString '\x03' },

    -- Paste with Ctrl+V (standard paste)
    { key = 'v', mods = 'CTRL', action = act.PasteFrom 'Clipboard' },
    -- Ctrl+Shift+V runs clip2path script in background (for images in Claude)
    {
        key = 'v',
        mods = 'CTRL|SHIFT',
        action = wezterm.action_callback(function(window, pane)
            local pane_id = pane:pane_id()
            os.execute('WEZTERM_PANE=' .. pane_id .. ' ' .. home .. '/bin/clip2path &')
        end)
    },

    -- Scroll
    { key = 'PageUp',   mods = 'SHIFT', action = act.ScrollByPage(-1) },
    { key = 'PageDown', mods = 'SHIFT', action = act.ScrollByPage(1) },

    -- Tab navigation
    { key = 'PageDown', mods = 'CTRL',  action = act.ActivateTabRelative(1) },
    { key = 'PageUp',   mods = 'CTRL',  action = act.ActivateTabRelative(-1) },

    -- New tab with current working directory (uses OSC 7)
    { key = 't',        mods = 'CTRL',  action = act.SpawnTab('CurrentPaneDomain') },

    -- Goto tab by number (Ctrl+1-9)
    { key = '1',        mods = 'CTRL',  action = act.ActivateTab(0) },
    { key = '2',        mods = 'CTRL',  action = act.ActivateTab(1) },
    { key = '3',        mods = 'CTRL',  action = act.ActivateTab(2) },
    { key = '4',        mods = 'CTRL',  action = act.ActivateTab(3) },
    { key = '5',        mods = 'CTRL',  action = act.ActivateTab(4) },
    { key = '6',        mods = 'CTRL',  action = act.ActivateTab(5) },
    { key = '7',        mods = 'CTRL',  action = act.ActivateTab(6) },
    { key = '8',        mods = 'CTRL',  action = act.ActivateTab(7) },
    { key = '9',        mods = 'CTRL',  action = act.ActivateTab(8) },

    -- Edit config (Ctrl+F2)
    {
        key = 'F2',
        mods = 'CTRL',
        action = act.SpawnCommandInNewTab {
            args = { os.getenv('EDITOR') or 'vim', wezterm.config_file },
        }
    },

    -- Reload config (Ctrl+F5)
    { key = 'F5',    mods = 'CTRL',       action = act.ReloadConfiguration },

    -- Shift+Enter inserts newline without submitting (Alt+Return)
    { key = 'Enter', mods = 'SHIFT',      action = act.SendString '\x1b\r' },

    -- Disable unicode input (Ctrl+Shift+U) - no_op equivalent
    { key = 'u',     mods = 'CTRL|SHIFT', action = act.Nop },
}

-- Mouse bindings
config.mouse_bindings = {
    -- Ctrl+Click to open URLs
    {
        event = { Up = { streak = 1, button = 'Left' } },
        mods = 'CTRL',
        action = act.OpenLinkAtMouseCursor,
    },
    -- Disable default click-to-open without modifier
    {
        event = { Up = { streak = 1, button = 'Left' } },
        mods = 'NONE',
        action = act.CompleteSelection 'ClipboardAndPrimarySelection',
    },
    -- Scroll speed (1 line per tick)
    {
        event = { Down = { streak = 1, button = { WheelUp = 1 } } },
        mods = 'NONE',
        action = act.ScrollByLine(-1),
    },
    {
        event = { Down = { streak = 1, button = { WheelDown = 1 } } },
        mods = 'NONE',
        action = act.ScrollByLine(1),
    },
}

return config
