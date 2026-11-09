//! Spring physics and build-time zero-runtime animation engine.
//!
//! Generates mathematically accurate damped harmonic oscillator keyframes
//! that run at 120 FPS on the browser compositor thread with zero JavaScript or WASM runtime overhead.

use std::fmt::Write;

/// Physical configuration for a spring oscillator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringConfig {
    /// Mass of the object in kilograms (typically 1.0).
    pub mass: f32,
    /// Spring stiffness constant k (higher = faster, snappier).
    pub stiffness: f32,
    /// Damping coefficient c (controls friction and bounce).
    pub damping: f32,
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self::GENTLE
    }
}

impl SpringConfig {
    /// Snappy spring for buttons, toggles, and popovers.
    pub const SNAPPY: Self = Self {
        mass: 1.0,
        stiffness: 220.0,
        damping: 22.0,
    };

    /// Gentle, natural spring for modals, sheets, and dialogs.
    pub const GENTLE: Self = Self {
        mass: 1.0,
        stiffness: 120.0,
        damping: 16.0,
    };

    /// Bouncy spring with noticeable overshoot for notifications and toasts.
    pub const BOUNCY: Self = Self {
        mass: 1.0,
        stiffness: 180.0,
        damping: 12.0,
    };

    /// Evaluates normalized position x(t) from t = 0 (rest at 0.0) to t -> infinity (settled at 1.0).
    ///
    /// Uses closed-form solution to the second-order linear differential equation:
    /// m * x''(t) + c * x'(t) + k * x(t) = 0
    #[allow(clippy::suboptimal_flops)]
    pub fn position_at(&self, t: f32) -> f32 {
        if t <= 0.0 {
            return 0.0;
        }

        let omega_n = (self.stiffness / self.mass).sqrt();
        let zeta = self.damping / (2.0 * (self.stiffness * self.mass).sqrt());

        if zeta < 1.0 {
            // Underdamped system (oscillations with decay)
            let omega_d = omega_n * (1.0 - zeta * zeta).sqrt();
            let decay = (-zeta * omega_n * t).exp();
            let envelope = decay
                * ((omega_d * t).cos() + (zeta / (1.0 - zeta * zeta).sqrt()) * (omega_d * t).sin());
            1.0 - envelope
        } else if (zeta - 1.0).abs() < 1e-4 {
            // Critically damped system (fastest settling without overshoot)
            let decay = (-omega_n * t).exp();
            1.0 - decay * (1.0 + omega_n * t)
        } else {
            // Overdamped system (slow sluggish return)
            let r1 = -omega_n * (zeta - (zeta * zeta - 1.0).sqrt());
            let r2 = -omega_n * (zeta + (zeta * zeta - 1.0).sqrt());
            let c1 = r2 / (r2 - r1);
            let c2 = -r1 / (r2 - r1);
            1.0 - (c1 * (r1 * t).exp() + c2 * (r2 * t).exp())
        }
    }

    /// Generates keyframe percentage samples for CSS `@keyframes`.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn generate_keyframes(&self, duration_secs: f32, steps: usize) -> Vec<(u32, f32)> {
        let mut samples = Vec::with_capacity(steps + 1);
        for i in 0..=steps {
            let pct = (i as f32 / steps as f32) * 100.0;
            let t = (i as f32 / steps as f32) * duration_secs;
            let val = self.position_at(t);
            samples.push((pct.round() as u32, val));
        }
        samples
    }

    /// Emits complete `@keyframes` CSS string for scale and opacity entrance.
    #[allow(clippy::suboptimal_flops)]
    pub fn to_scale_keyframes_css(&self, name: &str, duration_secs: f32) -> String {
        let mut css = String::new();
        let _ = writeln!(css, "@keyframes {} {{", name);
        let samples = self.generate_keyframes(duration_secs, 16);
        for (pct, val) in samples {
            let scale = 0.90 + (val * 0.10);
            let opacity = val.clamp(0.0, 1.0);
            let _ = writeln!(
                css,
                "  {}% {{ transform: scale({:.3}); opacity: {:.3}; }}",
                pct, scale, opacity
            );
        }
        let _ = writeln!(css, "}}");
        css
    }

    /// Emits complete `@keyframes` CSS string for translateY slide entrance.
    pub fn to_slide_keyframes_css(&self, name: &str, duration_secs: f32) -> String {
        let mut css = String::new();
        let _ = writeln!(css, "@keyframes {} {{", name);
        let samples = self.generate_keyframes(duration_secs, 16);
        for (pct, val) in samples {
            let translate_y = (1.0 - val) * 24.0;
            let opacity = val.clamp(0.0, 1.0);
            let _ = writeln!(
                css,
                "  {}% {{ transform: translateY({:.2}px); opacity: {:.3}; }}",
                pct, translate_y, opacity
            );
        }
        let _ = writeln!(css, "}}");
        css
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spring_physics_boundaries() {
        let spring = SpringConfig::GENTLE;
        assert!(spring.position_at(0.0).abs() < f32::EPSILON);

        let settled = spring.position_at(3.0);
        assert!((settled - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_spring_bouncy_overshoot() {
        let spring = SpringConfig::BOUNCY;
        // Bouncy spring should overshoot 1.0 around t = 0.2s - 0.4s
        let mut overshot = false;
        for i in 1..40 {
            let t = i as f32 * 0.02;
            if spring.position_at(t) > 1.02 {
                overshot = true;
                break;
            }
        }
        assert!(overshot, "Bouncy spring must exhibit noticeable overshoot");
    }

    #[test]
    fn test_keyframes_generation() {
        let spring = SpringConfig::SNAPPY;
        let css = spring.to_scale_keyframes_css("spring-in", 0.35);
        assert!(css.contains("@keyframes spring-in"));
        assert!(css.contains("transform: scale("));
        assert!(css.contains("opacity:"));
    }
}
