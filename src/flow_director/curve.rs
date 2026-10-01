//! Where a wave sits on the flow curve (`design/risk-flow-curve.png`):
//! engagement rises in a straight line from "failure is impossible" to the
//! balanced peak, then falls off steeply toward "failure is certain".
//!
//! The risk axis is [`wave_risk`](super::wave_risk): the share of the hp the
//! player brought into the wave that it took, 1 on death. So this reads only
//! mechanical pressure (aim, movement, timing). Decisions and reading enemies
//! do not show up in it.

/// PLACEHOLDER: the risk at the top of the curve. Half the hp at stake is
/// where agent playtesters rated waves as flow (zero damage read as boring).
pub const FLOW_PEAK: f32 = 0.5;
/// PLACEHOLDER: how fast engagement falls past the peak. At certain failure
/// it is e^-3, about 5% of the peak, as in the picture.
const FALLOFF: f32 = 3.0;
/// PLACEHOLDER: engagement from here up counts as a wave in flow.
pub const IN_FLOW: f32 = 0.6;

/// Engagement in `[0, 1]` for one wave's risk in `[0, 1]`.
pub fn engagement(risk: f32) -> f32 {
    let risk = risk.clamp(0.0, 1.0);
    if risk <= FLOW_PEAK {
        risk / FLOW_PEAK
    } else {
        (-FALLOFF * (risk - FLOW_PEAK) / (1.0 - FLOW_PEAK)).exp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rises_in_a_line_to_the_peak() {
        assert_eq!(engagement(0.0), 0.0);
        assert_eq!(engagement(FLOW_PEAK / 2.0), 0.5);
        assert_eq!(engagement(FLOW_PEAK), 1.0);
    }

    #[test]
    fn falls_steeply_past_the_peak_to_about_five_percent() {
        let certain = engagement(1.0);
        assert!((certain - (-3.0_f32).exp()).abs() < 1e-6);
        // Steeper on the danger side: the same distance from the peak costs more.
        assert!(engagement(FLOW_PEAK + 0.1) < engagement(FLOW_PEAK - 0.1));
    }

    #[test]
    fn in_flow_is_a_band_around_the_peak() {
        assert!(engagement(0.25) < IN_FLOW);
        assert!(engagement(0.35) >= IN_FLOW);
        assert!(engagement(0.55) >= IN_FLOW);
        assert!(engagement(0.65) < IN_FLOW);
    }
}
