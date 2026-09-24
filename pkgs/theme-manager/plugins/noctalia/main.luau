--!strict

return {
    load = function(ctx)
        local palette = ctx.inputs.palette
        if type(palette) ~= "string" then
            error("noctalia provider requires provider.inputs.palette")
        end

        local decoded = ctx.json.decode(palette)
        if type(decoded) ~= "table" then
            error("Noctalia palette JSON must contain an object")
        end
        return decoded
    end,
}
