--!strict

return {
    render = function(ctx)
        local role = ctx.settings.role or "compositor"
        local style = ctx.theme.roles[role]
        if style == nil then
            error("profile does not contain role '" .. role .. "'")
        end

        return {
            document = string.format(
                "profile=%s\nrole=%s\nopacity=%s\n",
                ctx.theme.profile,
                role,
                tostring(style.background_opacity)
            ),
        }
    end,
}
