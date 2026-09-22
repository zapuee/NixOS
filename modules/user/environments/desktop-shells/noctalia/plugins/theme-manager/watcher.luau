--!nonstrict

local paths = require("./lib/paths.luau")
local profiles = require("./lib/profiles.luau")
local profileState = require("./lib/profile_state.luau")
local pipeline = require("./structure/pipeline.luau")

-- Idle polling uses async reads, so the filesystem wait happens
-- outside the Luau worker. While applying, we temporarily tick at
-- the minimum supported interval so each small pipeline stage runs
-- quickly without exceeding Noctalia's per-callback CPU budget.
local IDLE_INTERVAL_MS = 100
local ACTIVE_INTERVAL_MS = 16

local activeProfile = nil
local lastObservedContents = nil
local readPending = false
local queuedContents = nil
local queuedReason = nil
local lastReadError = nil

local function profilePath(name)
    local path, err =
        profiles.path(
            paths.structureProfiles,
            name
        )

    if path == nil then
        noctalia.log(
            "Theme Manager watcher: "
            .. tostring(err)
        )
    end

    return path
end

local function publishStatus(status, message)
    noctalia.state.set(
        "structure_watcher_status",
        status
    )

    noctalia.state.set(
        "structure_watcher_message",
        message
    )
end

local function setActiveProfile(name, persist)
    if not profiles.isValidName(name) then
        return false
    end

    local path = profilePath(name)

    if path == nil or not noctalia.fileExists(path) then
        noctalia.log(
            "Theme Manager watcher: structure profile '"
            .. tostring(name)
            .. "' does not exist"
        )
        return false
    end

    if activeProfile == name then
        return true
    end

    activeProfile = name
    lastObservedContents = nil
    queuedContents = nil
    queuedReason = nil
    lastReadError = nil

    if persist then
        local ok, err =
            profileState.writeActiveStructure(name)

        if not ok then
            noctalia.log(
                "Theme Manager watcher: could not persist active profile: "
                .. tostring(err)
            )
        end
    end

    noctalia.log(
        "Theme Manager watcher: active profile = '"
        .. name
        .. "'"
    )

    publishStatus(
        "pending",
        "Waiting to apply " .. name
    )

    return true
end

local function restoreActiveProfile()
    local saved =
        profileState.readActiveStructure()

    if saved ~= nil and setActiveProfile(saved, false) then
        return
    end

    -- Nice first-run behavior: when there is exactly one structure
    -- profile, make it active automatically.
    local available, err =
        profiles.list(paths.structureProfiles)

    if err == nil and #available == 1 then
        setActiveProfile(available[1], true)
    end
end

local function queueRead()
    if
        activeProfile == nil
        or readPending
        or pipeline.isBusy()
        or queuedContents ~= nil
    then
        return
    end

    local path = profilePath(activeProfile)

    if path == nil then
        return
    end

    readPending = true

    -- Capture the profile associated with this async request. If the
    -- user switches profiles before the read completes, discard the
    -- stale result instead of applying old contents to the new profile.
    local requestedProfile = activeProfile

    local accepted =
        noctalia.readFileAsync(
            path,
            function(contents, err)
                readPending = false

                if requestedProfile ~= activeProfile then
                    return
                end

                if contents == nil then
                    local message = tostring(err)

                    if message ~= lastReadError then
                        noctalia.log(
                            "Theme Manager watcher: read failed for '"
                            .. tostring(activeProfile)
                            .. "': "
                            .. message
                        )
                        lastReadError = message
                    end

                    return
                end

                lastReadError = nil

                if contents == lastObservedContents then
                    return
                end

                local reason =
                    lastObservedContents == nil
                    and "initial/selected profile"
                    or "profile file changed"

                lastObservedContents = contents
                queuedContents = contents
                queuedReason = reason
            end
        )

    if not accepted then
        readPending = false
    end
end

local function startQueuedApply()
    if queuedContents == nil or pipeline.isBusy() then
        return false
    end

    local contents = queuedContents
    local reason = queuedReason

    queuedContents = nil
    queuedReason = nil

    local ok, err =
        pipeline.start(
            activeProfile,
            contents
        )

    if not ok then
        publishStatus("error", tostring(err))
        noctalia.log(
            "Theme Manager watcher: could not start apply: "
            .. tostring(err)
        )
        return false
    end

    noctalia.log(
        "Theme Manager watcher: applying '"
        .. tostring(activeProfile)
        .. "' ("
        .. tostring(reason)
        .. ")"
    )

    publishStatus(
        "applying",
        "Applying " .. tostring(activeProfile)
    )

    noctalia.setUpdateInterval(ACTIVE_INTERVAL_MS)
    return true
end

noctalia.state.watch(
    "active_structure_profile",
    function(value)
        if type(value) == "string" then
            setActiveProfile(value, true)
        end
    end
)

restoreActiveProfile()
noctalia.setUpdateInterval(IDLE_INTERVAL_MS)

noctalia.log(
    "Theme Manager watcher: service loaded"
)

function update()
    if pipeline.isBusy() then
        local status, message = pipeline.step()

        if status == "done" then
            noctalia.log(
                "Theme Manager watcher: applied '"
                .. tostring(message)
                .. "'"
            )

            publishStatus(
                "ok",
                "Applied " .. tostring(message)
            )

            noctalia.setUpdateInterval(IDLE_INTERVAL_MS)
        elseif status == "error" then
            noctalia.log(
                "Theme Manager watcher: apply failed: "
                .. tostring(message)
            )

            publishStatus("error", tostring(message))
            noctalia.notifyError(
                "Theme Manager",
                tostring(message)
            )

            noctalia.setUpdateInterval(IDLE_INTERVAL_MS)
        end

        return
    end

    if startQueuedApply() then
        return
    end

    queueRead()
end

function onIpc(event, payload)
    if event == "status" then
        noctalia.log(
            "Theme Manager watcher: active="
            .. tostring(activeProfile)
            .. ", busy="
            .. tostring(pipeline.isBusy())
            .. ", readPending="
            .. tostring(readPending)
        )
        return
    end

    if event == "apply" then
        -- Force the next async read to count as a change.
        lastObservedContents = nil
        return
    end
end

