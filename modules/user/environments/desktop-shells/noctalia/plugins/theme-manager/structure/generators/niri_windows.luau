--!nonstrict

local resolver = require("../resolver.luau")

local generator = {}


-- ============================================================
-- WINDOW RULE
-- ============================================================

local function writeWindowRule(
    lines,
    appId,
    radius,
    blur,
    opacity
)
    if
        appId == nil
        and radius == nil
        and blur ~= true
        and opacity == nil
    then
        return
    end


    table.insert(lines, "window-rule {")


    if appId ~= nil then
        table.insert(
            lines,
            "    match app-id=\""
                .. appId
                .. "\""
        )
    end


    if radius ~= nil then
        table.insert(
            lines,
            "    geometry-corner-radius "
                .. tostring(radius)
        )

        table.insert(
            lines,
            "    clip-to-geometry true"
        )
    end


    if opacity ~= nil then
        table.insert(
            lines,
            "    opacity "
                .. resolver.float(opacity)
        )
    end


    if blur == true then
        table.insert(
            lines,
            "    background-effect {"
        )

        table.insert(
            lines,
            "        blur true"
        )

        table.insert(
            lines,
            "    }"
        )
    end


    table.insert(lines, "}")
    table.insert(lines, "")
end


-- ============================================================
-- RESOLVERS
-- ============================================================

local function resolveRadius(
    variables,
    selector,
    propertyName
)
    local radius, err =
        resolver.tier(
            variables,
            selector,
            "corner_radius",
            propertyName
        )


    if err ~= nil then
        return nil, err
    end


    return resolver.number(
        radius,
        propertyName,
        0,
        nil
    )
end


local function resolveOpacity(
    variables,
    selector,
    propertyName
)
    local opacity, err =
        resolver.tier(
            variables,
            selector,
            "bg_opacity",
            propertyName
        )


    if err ~= nil then
        return nil, err
    end


    return resolver.number(
        opacity,
        propertyName,
        0,
        1
    )
end


-- ============================================================
-- GENERATE
-- ============================================================

function generator.generate(profile)
    local variables =
        profile.variables or {}

    local applications =
        profile.applications or {}

    local niri =
        applications.niri or {}

    local lines = {}

    local err


    -- ========================================================
    -- GLOBAL NIRI
    -- ========================================================

    if niri.enabled == false then
        return "", nil
    end


    local radius

    radius, err =
        resolveRadius(
            variables,
            niri.corner_radius,
            "Niri corner radius"
        )


    if err ~= nil then
        return nil, err
    end


    writeWindowRule(
        lines,
        nil,
        radius,
        false,
        nil
    )


    -- Prevent a solid backing rectangle from showing through
    -- translucent windows behind the focus ring.

    table.insert(
        lines,
        "window-rule {"
    )

    table.insert(
        lines,
        "    draw-border-with-background false"
    )

    table.insert(
        lines,
        "}"
    )

    table.insert(
        lines,
        ""
    )


    -- ========================================================
    -- TERMINAL
    -- ========================================================

    local terminal =
        applications.terminal


    if
        terminal ~= nil
        and terminal.enabled ~= false
        and terminal.target == "foot"
    then

        local terminalRadius

        terminalRadius, err =
            resolveRadius(
                variables,
                terminal.corner_radius,
                "Terminal corner radius"
            )


        if err ~= nil then
            return nil, err
        end


        local blur

        blur, err =
            resolver.bool(
                terminal.blur,
                "Terminal blur"
            )


        if err ~= nil then
            return nil, err
        end


        local _, materialErr =
            resolver.material(
                variables,
                terminal.material,
                "Terminal material"
            )


        if materialErr ~= nil then
            return nil, materialErr
        end


        -- Foot handles its own background alpha through foot.ini,
        -- so do not apply Niri compositor opacity here.

        writeWindowRule(
            lines,
            "^(foot|footclient)$",
            terminalRadius,
            blur,
            nil
        )
    end


    -- ========================================================
    -- FIREFOX
    -- ========================================================

    local firefox =
        applications.firefox


    if
        firefox ~= nil
        and firefox.enabled ~= false
    then

        local firefoxRadius

        firefoxRadius, err =
            resolveRadius(
                variables,
                firefox.corner_radius,
                "Firefox corner radius"
            )


        if err ~= nil then
            return nil, err
        end


        local firefoxOpacity

        firefoxOpacity, err =
            resolveOpacity(
                variables,
                firefox.background_opacity,
                "Firefox background opacity"
            )


        if err ~= nil then
            return nil, err
        end


        local blur

        blur, err =
            resolver.bool(
                firefox.blur,
                "Firefox blur"
            )


        if err ~= nil then
            return nil, err
        end


        local _, materialErr =
            resolver.material(
                variables,
                firefox.material,
                "Firefox material"
            )


        if materialErr ~= nil then
            return nil, materialErr
        end


        writeWindowRule(
            lines,
            "^firefox$",
            firefoxRadius,
            blur,
            firefoxOpacity
        )
    end


    return table.concat(
        lines,
        "\n"
    ), nil
end


return generator
