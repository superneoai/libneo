use libneo_gpui::appearance::{
    Appearance, SystemColor, reduce_motion_enabled, reduce_transparency_enabled,
    resolve_system_color,
};

fn main() {
    for appearance in [Appearance::Aqua, Appearance::DarkAqua] {
        for color in [
            SystemColor::LabelColor,
            SystemColor::WindowBackgroundColor,
            SystemColor::ControlAccentColor,
        ] {
            let resolved = resolve_system_color(color, appearance);
            println!(
                "{appearance:?} {color:?}: rgba({:.6}, {:.6}, {:.6}, {:.6})",
                resolved.r, resolved.g, resolved.b, resolved.a
            );
        }
    }

    println!("Reduce Transparency: {}", reduce_transparency_enabled());
    println!("Reduce Motion: {}", reduce_motion_enabled());
}
