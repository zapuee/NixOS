--!nonstrict

local strings = require("../lib/strings.luau")

local parser = {}

local function parseValue(rawValue)
    local value = strings.trim(rawValue)

    if value == "true" then
        return true, nil
    end

    if value == "false" then
        return false, nil
    end

    if
        #value >= 2
        and value:sub(1, 1) == "\""
        and value:sub(-1) == "\""
    then
        return value:sub(2, -2), nil
    end

    local number = tonumber(value)

    if number ~= nil then
        return number, nil
    end

    return nil,
        "Unsupported TOML value '"
        .. value
        .. "'"
end

local function getSection(root, path)
    local current = root

    for part in path:gmatch("[^%.]+") do
        local existing = current[part]

        if existing == nil then
            existing = {}
            current[part] = existing
        elseif type(existing) ~= "table" then
            return nil,
                "TOML section conflicts with key '"
                .. part
                .. "'"
        end

        current = existing
    end

    return current, nil
end

local function processLine(state, rawLine)
    local line = rawLine:gsub("\r$", "")
    line = strings.trim(line)

    if line == "" or line:sub(1, 1) == "#" then
        return true, nil
    end

    local section =
        line:match("^%[([^%]]+)%]$")

    if section ~= nil then
        local nextSection, err =
            getSection(state.result, section)

        if nextSection == nil then
            return false, err
        end

        state.current = nextSection
        return true, nil
    end

    local key, rawValue =
        line:match(
            "^([%w_%-]+)%s*=%s*(.-)%s*$"
        )

    if key == nil then
        return false,
            "Unsupported TOML syntax on line "
            .. tostring(state.line)
            .. ": "
            .. line
    end

    local value, valueErr =
        parseValue(rawValue)

    if valueErr ~= nil then
        return false,
            "Line "
            .. tostring(state.line)
            .. ": "
            .. valueErr
    end

    state.current[key] = value

    return true, nil
end

function parser.new(contents)
    local result = {}

    return {
        contents = contents or "",
        position = 1,
        line = 0,
        result = result,
        current = result,
        done = false,
        error = nil,
    }
end

-- Parse only a bounded number of lines per service tick. This keeps
-- Noctalia's per-callback CPU budget comfortably below its limit.
function parser.step(state, maxLines)
    if state.done then
        return true, state.error
    end

    local contents = state.contents
    local length = #contents
    local processed = 0

    while
        processed < maxLines
        and state.position <= length
    do
        local newline =
            string.find(
                contents,
                "\n",
                state.position,
                true
            )

        local line

        if newline ~= nil then
            line = contents:sub(
                state.position,
                newline - 1
            )

            state.position = newline + 1
        else
            line = contents:sub(state.position)
            state.position = length + 1
        end

        state.line = state.line + 1
        processed = processed + 1

        local ok, err = processLine(state, line)

        if not ok then
            state.done = true
            state.error = err
            return true, err
        end
    end

    if state.position > length then
        state.done = true
    end

    return state.done, state.error
end

return parser
