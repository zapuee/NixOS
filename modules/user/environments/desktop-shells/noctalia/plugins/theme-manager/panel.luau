--!nonstrict

local function render()
    panel.render(
        ui.column({
            gap = 16,
        }, {
            ui.row({
                align = "center",
            }, {
                ui.label({
                    text = "Theme Manager",
                    fontSize = 18,
                    fontWeight = "bold",
                    color = "primary",
                    flexGrow = 1,
                }),

                ui.button({
                    glyph = "close",
                    onClick = "onClose",
                }),
            }),

            ui.separator({}),

            ui.label({
                text = "Noctalia Profile",
                fontWeight = "bold",
            }),

            ui.select({
                options = {
                    "Napoleon",
                },
                selectedIndex = 0,
                onChange = "onNoctaliaProfileChanged",
            }),

            ui.label({
                text = "Structure Profile",
                fontWeight = "bold",
            }),

            ui.select({
                options = {
                    "Glass",
                },
                selectedIndex = 0,
                onChange = "onStructureProfileChanged",
            }),
        })
    )
end


function onOpen()
    render()
end


function onClose()
    panel.close()
end


function onNoctaliaProfileChanged(index, value)
    noctalia.log("Noctalia profile: " .. tostring(value))
end


function onStructureProfileChanged(index, value)
    noctalia.log("Structure profile: " .. tostring(value))
end
