use crate::config::{ApplicationWire, NiriWindow, NoctaliaExecution, OpacityOwner};
use anyhow::{Context, Result, bail};
use theme_core::{Cadence, Capability, CapabilityClaim, ResolvedRole, ResolvedTheme};

#[derive(Debug, Clone)]
pub struct RenderedArtifact {
    pub wire: String,
    pub relative_path: String,
    pub contents: String,
}

pub fn claims(wire_name: &str, wire: &ApplicationWire) -> Result<Vec<CapabilityClaim>> {
    let owner = format!("theme-manager-wire:{wire_name}");
    Ok(match wire {
        ApplicationWire::Niri { windows, .. } => {
            let mut claims = [
                (Capability::Gaps, "niri.layout"),
                (Capability::FocusRing, "niri.layout.focus-ring"),
                (Capability::Shadow, "niri.layout.shadow"),
                (Capability::CornerRadius, "niri.window-rule.global"),
            ]
            .into_iter()
            .map(|(capability, boundary)| CapabilityClaim {
                application: "niri".into(),
                capability,
                owner: owner.clone(),
                source: "built-in:niri".into(),
                cadence: Cadence::Runtime,
                boundary: boundary.into(),
                provides: true,
            })
            .collect::<Vec<_>>();
            claims.extend(windows.iter().map(|window| CapabilityClaim {
                application: window.application.clone(),
                capability: Capability::WindowStructure,
                owner: owner.clone(),
                source: "built-in:niri".into(),
                cadence: Cadence::Runtime,
                boundary: format!("niri.window-rule.{}", window.application),
                provides: true,
            }));
            claims
        }
        ApplicationWire::Foot { .. } => vec![CapabilityClaim {
            application: "foot".into(),
            capability: Capability::BackgroundAlpha,
            owner,
            source: "built-in:foot".into(),
            cadence: Cadence::Runtime,
            boundary: "foot.include.theme-manager.alpha".into(),
            provides: true,
        }],
        ApplicationWire::NoctaliaTemplate {
            execution,
            template,
            output,
            ..
        } => {
            if *execution == NoctaliaExecution::Headless {
                headless_template_claims(template, output.as_deref(), owner)?
            } else {
                shell_template_claims(template, owner)?
            }
        }
    })
}

fn headless_template_claims(
    template: &str,
    output: Option<&str>,
    owner: String,
) -> Result<Vec<CapabilityClaim>> {
    let boundary = format!(
        "theme-manager.generated:{}",
        output.context("validated headless output is missing")?
    );
    let source = format!("noctalia-template:{template}");
    let make = |application: &str, capability| CapabilityClaim {
        application: application.into(),
        capability,
        owner: owner.clone(),
        source: source.clone(),
        cadence: Cadence::Runtime,
        boundary: boundary.clone(),
        // Rendering a contained file does not prove that the application
        // consumes it. A future typed include/loader integration can promote
        // this trusted support metadata to a provided claim.
        provides: false,
    };
    Ok(match template {
        "repository:palette" => Vec::new(),
        "repository:foot-colors" => vec![
            make("foot", Capability::Colors),
            make("foot", Capability::CursorColor),
            make("foot", Capability::SelectionColor),
        ],
        "repository:pywalfox" => vec![make("firefox", Capability::Colors)],
        _ => bail!("unaudited headless template '{template}'"),
    })
}

pub fn headless_template(template: &str) -> Result<&'static str> {
    match template {
        "repository:palette" => Ok(include_str!("../../../integrations/noctalia/palette.json")),
        "repository:foot-colors" => Ok(include_str!("../../../integrations/noctalia/foot.ini")),
        "repository:pywalfox" => Ok(include_str!("../../../integrations/noctalia/pywalfox.json")),
        _ => bail!("unaudited headless template '{template}'"),
    }
}

fn shell_template_claims(template: &str, owner: String) -> Result<Vec<CapabilityClaim>> {
    let source = format!("noctalia-template:{template}");
    let make = |application: &str, capability, boundary: &str| CapabilityClaim {
        application: application.into(),
        capability,
        owner: owner.clone(),
        source: source.clone(),
        cadence: Cadence::Runtime,
        boundary: boundary.into(),
        provides: true,
    };
    Ok(match template {
        "user:foot-colors" => vec![
            make("foot", Capability::Colors, "foot.theme.noctalia.colors"),
            make(
                "foot",
                Capability::CursorColor,
                "foot.theme.noctalia.cursor",
            ),
            make(
                "foot",
                Capability::SelectionColor,
                "foot.theme.noctalia.selection",
            ),
        ],
        "builtin:starship" => vec![make(
            "starship",
            Capability::Colors,
            "starship.palette.noctalia",
        )],
        "user:pywalfox" => vec![make(
            "firefox",
            Capability::Colors,
            "firefox.pywalfox.colors",
        )],
        _ => bail!("unaudited shell template '{template}'"),
    })
}

pub fn render(
    wire_name: &str,
    wire: &ApplicationWire,
    theme: &ResolvedTheme,
) -> Result<Option<RenderedArtifact>> {
    match wire {
        ApplicationWire::Niri {
            global_role,
            windows,
            ..
        } => Ok(Some(RenderedArtifact {
            wire: wire_name.into(),
            relative_path: "niri.kdl".into(),
            contents: render_niri(theme, global_role, windows)?,
        })),
        ApplicationWire::Foot { role, .. } => Ok(Some(RenderedArtifact {
            wire: wire_name.into(),
            relative_path: "foot.ini".into(),
            contents: render_foot(theme, role)?,
        })),
        ApplicationWire::NoctaliaTemplate { .. } => Ok(None),
    }
}

fn render_foot(theme: &ResolvedTheme, role_name: &str) -> Result<String> {
    let role = theme
        .roles
        .get(role_name)
        .with_context(|| format!("Foot wire references missing role '{role_name}'"))?;
    let opacity = role
        .background_opacity
        .with_context(|| format!("role '{role_name}' has no background_opacity"))?;
    if !(0.0..=1.0).contains(&opacity) {
        bail!("Foot background opacity must be between 0 and 1");
    }
    Ok(format!(
        "# {}\n# Active structure profile: {}\n\n[colors-dark]\nalpha={}\n\n[colors-light]\nalpha={}\n",
        theme_core::GENERATED_MARKER,
        theme.profile,
        number(opacity),
        number(opacity),
    ))
}

fn render_niri(theme: &ResolvedTheme, global_role: &str, windows: &[NiriWindow]) -> Result<String> {
    let mut lines = vec![
        format!("// {}", theme_core::GENERATED_MARKER),
        format!("// Active structure profile: {}", theme.profile),
        String::new(),
        "layout {".into(),
        format!("    gaps {}", number(theme.layout.gaps)),
    ];
    let strut = theme.layout.outer_gaps - theme.layout.gaps;
    lines.extend([
        "    struts {".into(),
        format!("        left {}", number(strut)),
        format!("        right {}", number(strut)),
        format!("        top {}", number(strut)),
        format!("        bottom {}", number(strut)),
        "    }".into(),
        "    focus-ring {".into(),
        "        on".into(),
        format!("        width {}", number(theme.focus_ring.width)),
        format!(
            "        active-color \"{}\"",
            rgba(
                &theme.focus_ring.active_color,
                theme.focus_ring.active_opacity
            )?
        ),
        format!(
            "        inactive-color \"{}\"",
            rgba(
                &theme.focus_ring.inactive_color,
                theme.focus_ring.inactive_opacity
            )?
        ),
        format!(
            "        urgent-color \"{}\"",
            rgba(
                &theme.focus_ring.urgent_color,
                theme.focus_ring.urgent_opacity
            )?
        ),
        "    }".into(),
        "    shadow {".into(),
        "        on".into(),
        format!(
            "        color \"{}\"",
            rgba(&theme.shadow.color, theme.shadow.strength)?
        ),
        "    }".into(),
        "}".into(),
        String::new(),
    ]);

    if let Some(radius) = theme
        .roles
        .get(global_role)
        .and_then(|role| role.corner_radius)
    {
        lines.extend([
            "window-rule {".into(),
            format!("    geometry-corner-radius {}", number(radius)),
            "    clip-to-geometry true".into(),
            "}".into(),
            String::new(),
        ]);
    }
    lines.extend([
        "window-rule {".into(),
        "    draw-border-with-background false".into(),
        "}".into(),
        String::new(),
    ]);
    for window in windows {
        if window.app_id.chars().any(char::is_control) {
            bail!("Niri app-id match may not contain control characters");
        }
        let role = theme.roles.get(&window.role).with_context(|| {
            format!(
                "Niri window binding references missing role '{}'",
                window.role
            )
        })?;
        append_window_rule(
            &mut lines,
            &window.app_id,
            role,
            window.opacity_owner == OpacityOwner::Compositor,
        );
    }
    Ok(format!("{}\n", lines.join("\n")))
}

fn append_window_rule(
    lines: &mut Vec<String>,
    app_id: &str,
    role: &ResolvedRole,
    use_opacity: bool,
) {
    lines.push("window-rule {".into());
    lines.push(format!(
        "    match app-id=\"{}\"",
        app_id.replace('\\', "\\\\").replace('"', "\\\"")
    ));
    if let Some(radius) = role.corner_radius {
        lines.push(format!("    geometry-corner-radius {}", number(radius)));
        lines.push("    clip-to-geometry true".into());
    }
    if use_opacity && let Some(opacity) = role.background_opacity {
        lines.push(format!("    opacity {}", float(opacity)));
    }
    if role.blur == Some(true) {
        lines.extend([
            "    background-effect {".into(),
            "        blur true".into(),
            "    }".into(),
        ]);
    }
    lines.push("}".into());
    lines.push(String::new());
}

fn number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}

fn float(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.1}")
    } else {
        value.to_string()
    }
}

fn rgba(color: &str, opacity: f64) -> Result<String> {
    let hex = color.strip_prefix('#').unwrap_or(color);
    if !matches!(hex.len(), 6 | 8) || !hex.chars().all(|value| value.is_ascii_hexdigit()) {
        bail!("Niri adapter expected a hex color, got '{color}'");
    }
    let component = |range| u8::from_str_radix(&hex[range], 16).expect("validated hex");
    let source_alpha = if hex.len() == 8 {
        f64::from(component(6..8)) / 255.0
    } else {
        1.0
    };
    Ok(format!(
        "rgba({}, {}, {}, {})",
        component(0..2),
        component(2..4),
        component(4..6),
        float((source_alpha * opacity).clamp(0.0, 1.0))
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use theme_core::{Palette, Profile, resolve};

    fn theme(profile: &str) -> ResolvedTheme {
        let profile: Profile = toml::from_str(match profile {
            "glass" => include_str!("../../../profiles/glass.toml"),
            "paper" => include_str!("../../../profiles/paper.toml"),
            _ => unreachable!(),
        })
        .unwrap();
        resolve::resolve(
            &profile,
            &Palette {
                colors: [
                    ("primary", "#80d998"),
                    ("outline", "#7c9598"),
                    ("error", "#ffb4ab"),
                    ("shadow", "#000000"),
                ]
                .into_iter()
                .map(|(key, value)| (key.into(), value.into()))
                .collect(),
            },
        )
        .unwrap()
    }

    #[test]
    fn foot_and_niri_preserve_v0_2_output() {
        for name in ["glass", "paper"] {
            let theme = theme(name);
            assert_eq!(
                render_foot(&theme, "terminal").unwrap(),
                include_str!(concat!("../tests/fixtures/v0.2/", "glass", "/foot.ini"))
                    .replace("glass", name)
                    .replace("0.25", if name == "glass" { "0.25" } else { "1" })
            );
        }
    }
}
