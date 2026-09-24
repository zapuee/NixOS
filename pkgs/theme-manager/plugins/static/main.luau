--!strict

return {
    load = function(ctx)
        local colors = ctx.settings.colors
        if type(colors) ~= "table" then
            error("static provider requires a [provider.settings.colors] table")
        end
        return colors
    end,
}
