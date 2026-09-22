--!nonstrict

local strings = {}

function strings.trim(value)
    if value == nil then
        return nil
    end

    return (
        tostring(value)
            :gsub("^%s+", "")
            :gsub("%s+$", "")
    )
end

return strings
