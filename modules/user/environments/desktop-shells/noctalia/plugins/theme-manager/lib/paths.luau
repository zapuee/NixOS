--!nonstrict

local paths = {}

paths.root =
    noctalia.expandPath(
        "~/.local/share/noctalia/theme-manager"
    )

paths.noctaliaProfiles =
    paths.root .. "/noctalia-profiles"

paths.structureProfiles =
    paths.root .. "/structure-profiles"

paths.activeNoctaliaSettings =
    noctalia.expandPath(
        "~/.local/state/noctalia/settings.toml"
    )

paths.output =
    noctalia.expandPath(
        "~/.config/theme-manager"
    )

return paths
