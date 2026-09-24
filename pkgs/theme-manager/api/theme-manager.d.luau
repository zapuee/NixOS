--!strict
-- Theme-Manager Luau API v1. Scripts return a provider or target module table.

export type Settings = { [string]: any }
export type Inputs = { [string]: string }
export type Json = {
    decode: (input: string) -> any,
    encode: (input: any) -> string,
}

export type ResolvedRole = {
    background_opacity: number?,
    blur: boolean?,
    corner_radius: number?,
    material: string?,
    accent_color: string?,
    cursor_color: string?,
    selection_color: string?,
}

export type ResolvedTheme = {
    schema: number,
    profile: string,
    description: string,
    layout: { gaps: number, outer_gaps: number },
    focus_ring: {
        width: number,
        active_opacity: number,
        inactive_opacity: number,
        urgent_opacity: number,
        active_color: string,
        inactive_color: string,
        urgent_color: string,
    },
    shadow: { strength: number, color: string },
    roles: { [string]: ResolvedRole },
}

export type ProviderModule = {
    load: (ctx: { settings: Settings, inputs: Inputs, json: Json }) -> { [string]: string },
}

export type TargetModule = {
    render: (ctx: { settings: Settings, theme: ResolvedTheme, json: Json }) -> { [string]: string },
}

return {}
