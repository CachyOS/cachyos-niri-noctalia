# cachyos-niri-settings

[**Niri**](https://niri-wm.github.io/niri/) is a scrolling wayland compositor build on Smithay.

This repository contains `Niri` specific settings, like `config.kdl` and `xdg-desktop-portal`.

Dependencies:

```Sh
niri xdg-desktop-portal-gnome
```

## Reposotiry tree

### Path [`.config/niri/`](./etc/skel/.config/niri/):

in [`config.kdl`](./etc/skel/.config/niri/) you can find all `includes` from directory [`cfg/`](./etc/skel/.config/niri/cfg/)

### Path [`.config/niri/cfg/`](./etc/skel/.config/niri/cfg/)

| config | purpose |
| ------ | ------- |
| [`animation.kdl`](./etc/skel/.config/niri/cfg/animation.kdl) | default animataions; beziers and springs       |
| [`input.kdl`](./etc/skel/.config/niri/cfg/input.kdl) | libinput related settings; xbk layouts and etc |
| [`keybinds.kdl`](./etc/skel/.config/niri/cfg/keybinds.kdl) | keybinds; like colomn focus, terminal spawn |
| [`layout.kdl`](./etc/skel/.config/niri/cfg/layout.kdl) | layout related settings; how `Niri` behaves |
| [`misc.kdl`](./etc/skel/.config/niri/cfg/misc.kdl) | ungrouped settings; blur, cursors, etc |
| [`rules.kdl`](./etc/skel/.config/niri/cfg/rules.kdl) | window rules; like blur for noctalia |

### Path [`.config/xdg-desktop-portal/`](./etc/skel/.config/xdg-desktop-portal/)

In `niri-portals.conf` you can configure `xdg-desktop-portal` directives for `niri`; Default portal already set

<br/>

Thank you for choosing CachyOS!
