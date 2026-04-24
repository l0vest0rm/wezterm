local wezterm = require 'wezterm'

local config = {}

local function basename(path)
  if not path or path == '' then
    return ''
  end

  path = path:gsub('[\\/]+$', '')
  local name = path:match('([^/\\]+)$')
  return name or path
end

wezterm.on('format-tab-title', function(tab, tabs, panes, cfg, hover, max_width)
  local pane = tab.active_pane
  local cwd = pane.current_working_dir
  local user_vars = pane.user_vars or {}
  local title = nil

  if user_vars.WEZTERM_TAB_CWD and user_vars.WEZTERM_TAB_CWD ~= '' then
    title = user_vars.WEZTERM_TAB_CWD
  elseif cwd then
    if type(cwd) == 'table' and cwd.file_path then
      title = basename(cwd.file_path)
    elseif type(cwd) == 'string' then
      title = basename(cwd)
    end
  end

  if not title or title == '' then
    title = pane.title
  end

  return ' ' .. wezterm.truncate_right(title, math.max(max_width - 2, 1)) .. ' '
end)

return config
