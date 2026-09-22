--!nonstrict

local paths = require("./paths.luau")
local profiles = require("./profiles.luau")
local profileState = require("./profile_state.luau")

local actions = {}

function actions.applyNoctalia(name)
    local sourcePath, pathErr =
        profiles.path(paths.noctaliaProfiles, name)

    if sourcePath == nil then
        return false, pathErr
    end

    local contents, readErr =
        noctalia.readFile(sourcePath)

    if contents == nil then
        return false,
            "Could not read Noctalia profile '"
            .. tostring(name)
            .. "': "
            .. tostring(readErr)
    end

    local ok, writeErr =
        noctalia.writeFile(
            paths.activeNoctaliaSettings,
            contents
        )

    if not ok then
        return false,
            "Could not apply Noctalia profile '"
            .. tostring(name)
            .. "': "
            .. tostring(writeErr)
    end

    return true, nil
end

function actions.activateStructure(name)
    local sourcePath, pathErr =
        profiles.path(paths.structureProfiles, name)

    if sourcePath == nil then
        return false, pathErr
    end

    if not noctalia.fileExists(sourcePath) then
        return false,
            "Structure profile '"
            .. tostring(name)
            .. "' does not exist"
    end

    local ok, err =
        profileState.writeActiveStructure(name)

    if not ok then
        return false,
            "Could not activate structure profile '"
            .. tostring(name)
            .. "': "
            .. tostring(err)
    end

    -- Fast in-memory handoff. The marker above is the durable
    -- fallback when writing settings.toml reloads the plugin.
    noctalia.state.set(
        "active_structure_profile",
        name
    )

    return true, nil
end

return actions
