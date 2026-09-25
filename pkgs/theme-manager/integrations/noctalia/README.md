# Audited Noctalia templates

These repository-owned templates are registered explicitly by Home Manager.
They never fetch or execute community template code at runtime.

- `foot.ini` is derived from Noctalia 5.1.0's built-in Foot template. It writes
  only the included theme fragment and deliberately omits apply/undo hooks.
- `pywalfox.json` is derived from the `pywalfox-beta4` template in
  `noctalia-dev/community-templates`. Its only action is Noctalia's named
  `firefox-theme` post action.
- `palette.json` exports the normalized semantic palette consumed by Theme
  Manager.
