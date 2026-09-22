--!nonstrict

local strings = require("./strings.luau")
local profiles = require("./profiles.luau")

local profileState = {}

local dataDir, dataDirErr =
    noctalia.pluginDataDir()

local ACTIVE_STRUCTURE_FILE = nil

if dataDir ~= nil then
    ACTIVE_STRUCTURE_FILE =
        dataDir .. "/active-structure-profile.txt"
end

local function unavailableError()
    return "Theme Manager persistent data directory is unavailable: "
        .. tostring(dataDirErr)
end

function profileState.activeStructurePath()
    if ACTIVE_STRUCTURE_FILE == nil then
        return nil, unavailableError()
    end

    return ACTIVE_STRUCTURE_FILE, nil
end

function profileState.readActiveStructure()
    if ACTIVE_STRUCTURE_FILE == nil then
        return nil, unavailableError()
    end

    local contents, err =
        noctalia.readFile(ACTIVE_STRUCTURE_FILE)

    if contents == nil then
        return nil, err
    end

    local name = strings.trim(contents)

    if not profiles.isValidName(name) then
        return nil,
            "Invalid active structure profile marker"
    end

    return name, nil
end

function profileState.writeActiveStructure(name)
    if ACTIVE_STRUCTURE_FILE == nil then
        return false, unavailableError()
    end

    if not profiles.isValidName(name) then
        return false,
            "Invalid structure profile name: "
            .. tostring(name)
    end

    return noctalia.writeFile(
        ACTIVE_STRUCTURE_FILE,
        name .. "\n"
    )
end

return profileState
