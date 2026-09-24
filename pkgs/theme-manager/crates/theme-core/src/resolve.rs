use crate::{Palette, profile::*};
use anyhow::{Context, Result, bail};
use std::collections::BTreeMap;

fn bounded(value: f64, name: &str, min: f64, max: f64) -> Result<f64> {
    if !value.is_finite() || !(min..=max).contains(&value) {
        bail!("{name} must be between {min} and {max}, got {value}");
    }
    Ok(value)
}

fn non_negative(value: f64, name: &str) -> Result<f64> {
    if !value.is_finite() || value < 0.0 {
        bail!("{name} must be a finite non-negative number, got {value}");
    }
    Ok(value)
}

fn resolve_number(
    selector: &NumberRef,
    values: &BTreeMap<String, Numeric>,
    category: &str,
) -> Result<f64> {
    if let Some(value) = selector.literal() {
        return Ok(value);
    }

    let NumberRef::Ref(name) = selector else {
        unreachable!()
    };
    values
        .get(name)
        .copied()
        .map(Numeric::as_f64)
        .with_context(|| format!("unknown {category} tier '{name}'"))
}

fn resolve_material(selector: Option<&str>, preferred: &str) -> Option<String> {
    selector.map(|value| {
        if value == "preferred" {
            preferred.to_string()
        } else {
            value.to_string()
        }
    })
}

fn resolve_color(reference: &str, palette: &Palette) -> Result<String> {
    if reference.starts_with('#') {
        Palette::validate_color(reference)?;
        return Ok(reference.to_string());
    }

    Ok(palette.get(reference)?.to_string())
}

pub fn resolve(profile: &Profile, palette: &Palette) -> Result<ResolvedTheme> {
    if profile.schema != 1 {
        bail!("unsupported profile schema {}; expected 1", profile.schema);
    }

    palette.validate()?;

    let gaps = non_negative(profile.layout.gaps.as_f64(), "layout.gaps")?;
    let outer_gaps = non_negative(profile.layout.outer_gaps.as_f64(), "layout.outer_gaps")?;

    let focus_ring = ResolvedFocusRing {
        width: non_negative(profile.focus_ring.width.as_f64(), "focus_ring.width")?,
        active_opacity: bounded(
            profile.focus_ring.active_opacity.as_f64(),
            "focus_ring.active_opacity",
            0.0,
            1.0,
        )?,
        inactive_opacity: bounded(
            profile.focus_ring.inactive_opacity.as_f64(),
            "focus_ring.inactive_opacity",
            0.0,
            1.0,
        )?,
        urgent_opacity: bounded(
            profile.focus_ring.urgent_opacity.as_f64(),
            "focus_ring.urgent_opacity",
            0.0,
            1.0,
        )?,
        active_color: resolve_color(&profile.focus_ring.active_color, palette)?,
        inactive_color: resolve_color(&profile.focus_ring.inactive_color, palette)?,
        urgent_color: resolve_color(&profile.focus_ring.urgent_color, palette)?,
    };

    let shadow = ResolvedShadow {
        strength: bounded(
            profile.shadow.strength.as_f64(),
            "shadow.strength",
            0.0,
            1.0,
        )?,
        color: resolve_color(&profile.shadow.color, palette)?,
    };

    let mut roles = BTreeMap::new();
    for (name, role) in &profile.roles {
        let background_opacity = role
            .background_opacity
            .as_ref()
            .map(|value| {
                resolve_number(value, &profile.variables.opacity, "opacity")
                    .and_then(|v| bounded(v, &format!("roles.{name}.background_opacity"), 0.0, 1.0))
            })
            .transpose()?;

        let corner_radius = role
            .corner_radius
            .as_ref()
            .map(|value| {
                resolve_number(value, &profile.variables.radius, "radius")
                    .and_then(|v| non_negative(v, &format!("roles.{name}.corner_radius")))
            })
            .transpose()?;

        roles.insert(
            name.clone(),
            ResolvedRole {
                background_opacity,
                blur: role.blur,
                corner_radius,
                material: resolve_material(role.material.as_deref(), &profile.variables.material),
                accent_color: role
                    .accent_color
                    .as_deref()
                    .map(|v| resolve_color(v, palette))
                    .transpose()?,
                cursor_color: role
                    .cursor_color
                    .as_deref()
                    .map(|v| resolve_color(v, palette))
                    .transpose()?,
                selection_color: role
                    .selection_color
                    .as_deref()
                    .map(|v| resolve_color(v, palette))
                    .transpose()?,
            },
        );
    }

    Ok(ResolvedTheme {
        schema: profile.schema,
        profile: profile.name.clone(),
        description: profile.description.clone(),
        layout: ResolvedLayout { gaps, outer_gaps },
        focus_ring,
        shadow,
        roles,
    })
}

#[cfg(test)]
mod tests {
    use super::{bounded, non_negative};

    #[test]
    fn rejects_non_finite_numbers() {
        assert!(bounded(f64::NAN, "value", 0.0, 1.0).is_err());
        assert!(bounded(f64::INFINITY, "value", 0.0, 1.0).is_err());
        assert!(non_negative(f64::NAN, "value").is_err());
        assert!(non_negative(f64::INFINITY, "value").is_err());
    }
}
