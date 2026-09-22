--!nonstrict

local resolver = {}

local VALID_TIERS = {
    primary = true,
    secondary = true,
    tertiary = true,
}

local VALID_MATERIALS = {
    glass = true,
    solid = true,
}

local VALID_COLOR_ROLES = {
    primary = true,
    on_primary = true,
    secondary = true,
    on_secondary = true,
    tertiary = true,
    on_tertiary = true,
    error = true,
    on_error = true,
    surface = true,
    on_surface = true,
    surface_variant = true,
    on_surface_variant = true,
    outline = true,
    shadow = true,
    hover = true,
    on_hover = true,
}

function resolver.number(
    value,
    propertyName,
    minimum,
    maximum
)
    if value == nil then
        return nil, nil
    end

    if type(value) ~= "number" then
        return nil,
            propertyName .. " must be a number"
    end

    if minimum ~= nil and value < minimum then
        return nil,
            propertyName
            .. " must be at least "
            .. tostring(minimum)
    end

    if maximum ~= nil and value > maximum then
        return nil,
            propertyName
            .. " must be at most "
            .. tostring(maximum)
    end

    return value, nil
end

function resolver.tier(
    variables,
    selector,
    suffix,
    propertyName
)
    if selector == nil then
        return nil, nil
    end

    if type(selector) == "number" then
        return selector, nil
    end

    if type(selector) ~= "string" then
        return nil,
            propertyName
            .. " must be a tier or numeric value"
    end

    if not VALID_TIERS[selector] then
        return nil,
            propertyName
            .. " has invalid tier '"
            .. selector
            .. "'"
    end

    local variableName =
        selector .. "_" .. suffix

    local value = variables[variableName]

    if value == nil then
        return nil,
            propertyName
            .. " references undefined variable '"
            .. variableName
            .. "'"
    end

    return value, nil
end

function resolver.default(
    variables,
    variableName,
    selector,
    propertyName
)
    if selector == nil then
        return nil, nil
    end

    if selector == "default" then
        local value = variables[variableName]

        if value == nil then
            return nil,
                propertyName
                .. " references undefined variable '"
                .. variableName
                .. "'"
        end

        return value, nil
    end

    return selector, nil
end

function resolver.bool(value, propertyName)
    if value == nil then
        return nil, nil
    end

    if type(value) ~= "boolean" then
        return nil,
            propertyName .. " must be true or false"
    end

    return value, nil
end

function resolver.material(
    variables,
    selector,
    propertyName
)
    if selector == nil then
        return nil, nil
    end

    local value = selector

    if selector == "preferred" then
        value = variables.preferred_material

        if value == nil then
            return nil,
                propertyName
                .. " references undefined variable 'preferred_material'"
        end
    end

    if
        type(value) ~= "string"
        or not VALID_MATERIALS[value]
    then
        return nil,
            propertyName
            .. " must be 'glass' or 'solid'"
    end

    return value, nil
end

function resolver.colorRole(
    selector,
    propertyName
)
    if selector == nil then
        return nil, nil
    end

    if type(selector) ~= "string" then
        return nil,
            propertyName
            .. " must reference a Noctalia color"
    end

    local role =
        selector:match("^color%.([%w_]+)$")

    if role == nil then
        return nil,
            propertyName
            .. " must use the form 'color.<role>'"
    end

    if not VALID_COLOR_ROLES[role] then
        return nil,
            propertyName
            .. " references unknown Noctalia color role '"
            .. role
            .. "'"
    end

    return role, nil
end

function resolver.colorExpression(role, alpha)
    if alpha ~= nil then
        return "{{ colors."
            .. role
            .. ".default.rgba | set_alpha "
            .. tostring(alpha)
            .. " }}"
    end

    return "{{ colors."
        .. role
        .. ".default.hex }}"
end

return resolver
