# Local WezTerm Setup

This directory stores a repo-managed `wezterm.lua` that makes tab titles show
the basename of the active pane working directory.

Install it with:

```sh
make install-config
```

This copies the repo-managed config to `$HOME/.config/wezterm/wezterm.lua`
and also ensures your `~/.zshrc` sources a copied WezTerm shell integration
script from `$HOME/.config/wezterm/wezterm.sh`, plus a small helper script
from `$HOME/.config/wezterm/tab-title.sh` that publishes the current directory
basename as a user var, so tab titles reliably track `cd` changes.

This means the installed config does not depend on the repo staying in the
same path after installation.

If you also want to install the locally built macOS app bundle:

```sh
make app
make install
```
