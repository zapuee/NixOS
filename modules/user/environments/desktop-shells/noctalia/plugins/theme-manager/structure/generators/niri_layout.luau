--!nonstrict

local resolver = require("../resolver.luau")

local generator = {}

function generator.generate(profile)
    local variables = profile.variables or {}
    local applications = profile.applications or {}
    local niri = applications.niri or {}

    if niri.enabled == false then
        return "", nil
    end

    local gaps, err =
        resolver.default(
            variables,
            "gaps",
            niri.gaps,
            "Niri gaps"
        )

    if err ~= nil then
        return nil, err
    end

    gaps, err =
        resolver.number(
            gaps,
            "Niri gaps",
            0,
            nil
        )

    if err ~= nil then
        return nil, err
    end

    local outerGaps
    outerGaps, err =
        resolver.default(
            variables,
            "outer_gaps",
            niri.outer_gaps,
            "Niri outer gaps"
        )

    if err ~= nil then
        return nil, err
    end

    outerGaps, err =
        resolver.number(
            outerGaps,
            "Niri outer gaps",
            0,
            nil
        )

    if err ~= nil then
        return nil, err
    end

    if outerGaps ~= nil and gaps == nil then
        return nil,
            "Niri outer gaps requires Niri gaps"
    end

    local focusRingWidth
    focusRingWidth, err =
        resolver.default(
            variables,
            "focus_ring_width",
            niri.focus_ring_width,
            "Niri focus ring width"
        )

    if err ~= nil then
        return nil, err
    end

    focusRingWidth, err =
        resolver.number(
            focusRingWidth,
            "Niri focus ring width",
            0,
            nil
        )

    if err ~= nil then
        return nil, err
    end

    local shadowStrength
    shadowStrength, err =
        resolver.default(
            variables,
            "shadow_strength",
            niri.shadow_strength,
            "Niri shadow strength"
        )

    if err ~= nil then
        return nil, err
    end

    shadowStrength, err =
        resolver.number(
            shadowStrength,
            "Niri shadow strength",
            0,
            1
        )

    if err ~= nil then
        return nil, err
    end

    if
        gaps == nil
        and outerGaps == nil
        and focusRingWidth == nil
        and shadowStrength == nil
    then
        return "", nil
    end

    local lines = { "layout {" }

    if gaps ~= nil then
        table.insert(
            lines,
            "    gaps " .. tostring(gaps)
        )
    end

    if gaps ~= nil and outerGaps ~= nil then
        local strut = outerGaps - gaps

        table.insert(lines, "    struts {")
        table.insert(lines, "        left " .. tostring(strut))
        table.insert(lines, "        right " .. tostring(strut))
        table.insert(lines, "        top " .. tostring(strut))
        table.insert(lines, "        bottom " .. tostring(strut))
        table.insert(lines, "    }")
    end

    if focusRingWidth ~= nil then
        table.insert(lines, "    focus-ring {")
        table.insert(lines, "        on")
        table.insert(
            lines,
            "        width " .. tostring(focusRingWidth)
        )
        table.insert(lines, "    }")
    end

    if shadowStrength ~= nil then
        table.insert(lines, "    shadow {")
        table.insert(lines, "        on")
        table.insert(lines, "    }")
    end

    table.insert(lines, "}")
    table.insert(lines, "")

    return table.concat(lines, "\n"), nil
end

return generator
