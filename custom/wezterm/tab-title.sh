# shellcheck shell=bash

if [ -z "${BASH_VERSION-}" ] && [ -z "${ZSH_NAME-}" ]; then
  return 0
fi

__wezterm_tab_title_basename() {
  local path="${PWD%/}"
  if [ -z "$path" ]; then
    printf '%s\n' '/'
    return 0
  fi
  printf '%s\n' "${path##*/}"
}

__wezterm_tab_title_update() {
  if ! type __wezterm_set_user_var >/dev/null 2>&1; then
    return 0
  fi
  __wezterm_set_user_var WEZTERM_TAB_CWD "$(__wezterm_tab_title_basename)"
}

if [ -n "${ZSH_NAME-}" ]; then
  typeset -ga precmd_functions chpwd_functions
  if [[ " ${precmd_functions[*]} " != *" __wezterm_tab_title_update "* ]]; then
    precmd_functions+=(__wezterm_tab_title_update)
  fi
  if [[ " ${chpwd_functions[*]} " != *" __wezterm_tab_title_update "* ]]; then
    chpwd_functions+=(__wezterm_tab_title_update)
  fi
elif [ -n "${BASH_VERSION-}" ]; then
  declare -ga precmd_functions
  case " ${precmd_functions[*]-} " in
    *" __wezterm_tab_title_update "*) ;;
    *) precmd_functions+=(__wezterm_tab_title_update) ;;
  esac
fi

__wezterm_tab_title_update
