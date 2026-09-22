--!nonstrict

local profiles = {}

function profiles.isValidName(name)
    if type(name) ~= "string" or name == "" then
        return false
    end

    if name == "." or name == ".." then
        return false
    end

    if
        name:find("/", 1, true) ~= nil
        or name:find("\\", 1, true) ~= nil
    then
        return false
    end

    return true
end

function profiles.nameFromFilename(filename)
    if
        type(filename) == "string"
        and filename:sub(-5) == ".toml"
    then
        local name = filename:sub(1, -6)

        if profiles.isValidName(name) then
            return name
        end
    end

    return nil
end

function profiles.path(directory, name)
    if not profiles.isValidName(name) then
        return nil,
            "Invalid profile name: "
            .. tostring(name)
    end

    return directory .. "/" .. name .. ".toml", nil
end

function profiles.list(directory)
    local entries, err = noctalia.listDir(directory)

    if entries == nil then
        return {}, err
    end

    local result = {}

    for _, filename in ipairs(entries) do
        local name = profiles.nameFromFilename(filename)

        if name ~= nil then
            table.insert(result, name)
        end
    end

    table.sort(result)

    return result, nil
end

function profiles.indexOf(list, value)
    if value == nil then
        return nil
    end

    for index, item in ipairs(list) do
        if item == value then
            -- ui.select uses zero-based indexes.
            return index - 1
        end
    end

    return nil
end

return profiles
