--!nonstrict

local paths = require("./lib/paths.luau")
local profiles = require("./lib/profiles.luau")
local profileState = require("./lib/profile_state.luau")
local profileActions = require("./lib/profile_actions.luau")

local noctaliaProfiles = {}
local structureProfiles = {}

local selectedNoctaliaIndex = 0
local selectedStructureIndex = 0

local selectedNoctaliaProfile = nil
local selectedStructureProfile = nil

local statusText = "Profiles have not been loaded yet."
local isOpen = false

local function setStatus(text)
    statusText = tostring(text)
end

local function refreshProfiles()
    local noctaliaResult, noctaliaErr =
        profiles.list(paths.noctaliaProfiles)

    if noctaliaErr ~= nil then
        setStatus(
            "Failed to read Noctalia profiles: "
            .. tostring(noctaliaErr)
        )
        noctalia.notifyError("Theme Manager", statusText)
        return false
    end

    local structureResult, structureErr =
        profiles.list(paths.structureProfiles)

    if structureErr ~= nil then
        setStatus(
            "Failed to read structure profiles: "
            .. tostring(structureErr)
        )
        noctalia.notifyError("Theme Manager", statusText)
        return false
    end

    noctaliaProfiles = noctaliaResult
    structureProfiles = structureResult

    if #noctaliaProfiles == 0 then
        selectedNoctaliaIndex = 0
        selectedNoctaliaProfile = nil
    else
        local existing =
            profiles.indexOf(
                noctaliaProfiles,
                selectedNoctaliaProfile
            )

        if existing == nil then
            selectedNoctaliaIndex = 0
            selectedNoctaliaProfile = noctaliaProfiles[1]
        else
            selectedNoctaliaIndex = existing
        end
    end

    if #structureProfiles == 0 then
        selectedStructureIndex = 0
        selectedStructureProfile = nil
    else
        local active =
            profileState.readActiveStructure()

        local preferred =
            selectedStructureProfile or active

        local existing =
            profiles.indexOf(
                structureProfiles,
                preferred
            )

        if existing == nil then
            selectedStructureIndex = 0
            selectedStructureProfile = structureProfiles[1]
        else
            selectedStructureIndex = existing
            selectedStructureProfile = preferred
        end
    end

    setStatus(
        "Found "
        .. tostring(#noctaliaProfiles)
        .. " Noctalia profile(s) and "
        .. tostring(#structureProfiles)
        .. " structure profile(s)."
    )

    return true
end

local function applyProfiles()
    if selectedNoctaliaProfile == nil then
        setStatus("No Noctalia profile selected.")
        noctalia.notifyError("Theme Manager", statusText)
        return false
    end

    if selectedStructureProfile == nil then
        setStatus("No structure profile selected.")
        noctalia.notifyError("Theme Manager", statusText)
        return false
    end

    -- Persist structure selection first so a Noctalia config reload
    -- cannot lose it.
    local structureOk, structureErr =
        profileActions.activateStructure(
            selectedStructureProfile
        )

    if not structureOk then
        setStatus(structureErr)
        noctalia.notifyError("Theme Manager", statusText)
        return false
    end

    local noctaliaOk, noctaliaErr =
        profileActions.applyNoctalia(
            selectedNoctaliaProfile
        )

    if not noctaliaOk then
        setStatus(noctaliaErr)
        noctalia.notifyError("Theme Manager", statusText)
        return false
    end

    setStatus(
        "Applied Noctalia profile '"
        .. selectedNoctaliaProfile
        .. "' and activated structure profile '"
        .. selectedStructureProfile
        .. "'."
    )

    noctalia.notify(
        "Theme Manager",
        "Applied "
        .. selectedNoctaliaProfile
        .. " + "
        .. selectedStructureProfile
    )

    return true
end

local function render()
    if not isOpen then
        return
    end

    local canApply =
        selectedNoctaliaProfile ~= nil
        and selectedStructureProfile ~= nil

    panel.render(
        ui.column(
            {
                gap = 14,
                padding = 16,
            },
            {
                ui.row(
                    {
                        align = "center",
                        justify = "space_between",
                    },
                    {
                        ui.label({
                            text = "Theme Manager",
                            fontSize = 18,
                            fontWeight = "bold",
                            color = "primary",
                            flexGrow = 1,
                        }),
                        ui.button({
                            glyph = "close",
                            glyphSize = 18,
                            variant = "ghost",
                            controlSize = "sm",
                            onClick = function()
                                panel.close()
                            end,
                        }),
                    }
                ),

                ui.separator({}),

                ui.label({
                    text = "Noctalia Profile",
                    fontWeight = "bold",
                }),
                ui.select({
                    options = noctaliaProfiles,
                    selectedIndex = selectedNoctaliaIndex,
                    placeholder = "No profiles found",
                    enabled = #noctaliaProfiles > 0,
                    onChange = function(index, value)
                        local nextIndex = tonumber(index)

                        if nextIndex == nil then
                            return
                        end

                        selectedNoctaliaIndex = nextIndex
                        selectedNoctaliaProfile =
                            value
                            or noctaliaProfiles[nextIndex + 1]

                        setStatus(
                            "Pending Noctalia profile: "
                            .. tostring(selectedNoctaliaProfile)
                        )
                        render()
                    end,
                }),

                ui.label({
                    text = "Structure Profile",
                    fontWeight = "bold",
                }),
                ui.select({
                    options = structureProfiles,
                    selectedIndex = selectedStructureIndex,
                    placeholder = "No profiles found",
                    enabled = #structureProfiles > 0,
                    onChange = function(index, value)
                        local nextIndex = tonumber(index)

                        if nextIndex == nil then
                            return
                        end

                        selectedStructureIndex = nextIndex
                        selectedStructureProfile =
                            value
                            or structureProfiles[nextIndex + 1]

                        setStatus(
                            "Pending structure profile: "
                            .. tostring(selectedStructureProfile)
                        )
                        render()
                    end,
                }),

                ui.button({
                    text = "Apply",
                    glyph = "check",
                    variant = "primary",
                    enabled = canApply,
                    onClick = function()
                        applyProfiles()
                        render()
                    end,
                }),

                ui.separator({}),

                ui.label({
                    text = statusText,
                    color = "on_surface_variant",
                    maxLines = 4,
                }),

                ui.button({
                    text = "Refresh Profiles",
                    glyph = "refresh",
                    variant = "secondary",
                    onClick = function()
                        refreshProfiles()
                        render()
                    end,
                }),
            }
        )
    )
end

noctalia.state.watch(
    "structure_watcher_message",
    function(value)
        if value ~= nil then
            setStatus(value)
            render()
        end
    end
)

function onOpen(context)
    isOpen = true
    refreshProfiles()
    render()
end

function onClose()
    isOpen = false
end
