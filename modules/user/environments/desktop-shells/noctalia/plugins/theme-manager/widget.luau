--!nonstrict

noctalia.setUpdateInterval(60000)

function update()
    barWidget.setGlyph("palette")
    barWidget.setTooltip("Theme Manager")
end

function onClick()
    noctalia.togglePanel("zap/theme-manager:panel")
end
