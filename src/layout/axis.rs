use super::{LABEL_LINE, LABEL_SIZE, count, fit_text, title::two_lines};
use crate::{
    error::ChartWarning,
    metrics::TextMetrics,
    scene::{Element, Text, TextAnchor},
    spec::{ChartSpec, Locale, NumberStyle, ValueFormat},
};

/// Centers the category axis title below a plot whose categories run horizontally.
pub(crate) fn add_bottom_category_title(
    spec: &ChartSpec,
    left: f64,
    plot_width: f64,
    chart_height: f64,
    elements: &mut Vec<Element>,
    warnings: &mut Vec<ChartWarning>,
    metrics: &impl TextMetrics,
) {
    if let Some(title) = &spec.category_axis.title {
        let title = fit_text(
            title,
            plot_width,
            LABEL_SIZE,
            metrics,
            warnings,
            "/categoryAxis/title",
        );
        elements.push(Element::Text(Text {
            x: left + plot_width / 2.0,
            y: chart_height - 14.0,
            class: "chartlet-axis-title",
            anchor: TextAnchor::Middle,
            content: title,
        }));
    }
}

/// The lines of a category label: one if it fits `max_width`, otherwise two, broken at a space,
/// before anything is shortened.
fn label_lines(
    label: &str,
    max_width: f64,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
    path: &str,
) -> Vec<String> {
    match two_lines(label, max_width, LABEL_SIZE, metrics) {
        Some((first, rest)) => vec![
            first.to_owned(),
            fit_text(rest, max_width, LABEL_SIZE, metrics, warnings, path),
        ],
        None => vec![fit_text(
            label, max_width, LABEL_SIZE, metrics, warnings, path,
        )],
    }
}

/// Draws the label of a category below a plot whose categories run horizontally, centered at
/// `x` with its first baseline at `y`, on up to two lines, see [`label_lines`].
pub(crate) fn push_category_label(
    elements: &mut Vec<Element>,
    label: &str,
    (x, y): (f64, f64),
    max_width: f64,
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
    path: &str,
) {
    let lines = label_lines(label, max_width, metrics, warnings, path);
    for (index, content) in lines.into_iter().enumerate() {
        elements.push(Element::Text(Text {
            x,
            y: y + count(index) * LABEL_LINE,
            class: "chartlet-label",
            anchor: TextAnchor::Middle,
            content,
        }));
    }
}

/// Draws the label of a category left of a plot whose categories run vertically, ending at `x`
/// and centered on the band around `center`. A band too narrow for two lines keeps the label on
/// one, shortened if need be; otherwise see [`label_lines`].
pub(crate) fn push_side_label(
    elements: &mut Vec<Element>,
    label: &str,
    (x, center): (f64, f64),
    (max_width, band): (f64, f64),
    metrics: &impl TextMetrics,
    warnings: &mut Vec<ChartWarning>,
    path: &str,
) {
    let lines = if band >= 2.0 * LABEL_LINE + 4.0 {
        label_lines(label, max_width, metrics, warnings, path)
    } else {
        vec![fit_text(
            label, max_width, LABEL_SIZE, metrics, warnings, path,
        )]
    };
    let first = center + 4.0 - count(lines.len() - 1) * LABEL_LINE / 2.0;
    for (index, content) in lines.into_iter().enumerate() {
        elements.push(Element::Text(Text {
            x,
            y: first + count(index) * LABEL_LINE,
            class: "chartlet-label",
            anchor: TextAnchor::End,
            content,
        }));
    }
}

/// Writes a value for reading: percent scaled and suffixed, fixed decimals if the style asks for
/// them, a true minus sign (U+2212, which screen readers announce as “minus”), and the locale's
/// decimal separator.
pub(crate) fn format_value(value: f64, style: impl Into<NumberStyle>) -> String {
    let style = style.into();
    let value = match style.format {
        ValueFormat::Number => value,
        ValueFormat::Percent => value * 100.0,
    };
    let digits = match style.decimals {
        Some(decimals) => format!("{:.*}", usize::from(decimals), tidy(value)),
        None => format!("{}", tidy(value)),
    };
    // Rounding to fixed decimals can turn a small negative value into `-0.0`.
    let digits = if digits.starts_with('-') && digits.trim_start_matches(['-', '0', '.']).is_empty()
    {
        digits[1..].to_owned()
    } else {
        digits
    };
    let digits = digits.replace('-', "\u{2212}");
    let digits = match style.locale {
        Locale::En => digits,
        Locale::De => digits.replace('.', ","),
    };
    let suffix = match (style.format, style.locale) {
        (ValueFormat::Number, _) => "",
        (ValueFormat::Percent, Locale::En) => "%",
        (ValueFormat::Percent, Locale::De) => "\u{202f}%",
    };
    format!("{digits}{suffix}")
}

/// Writes an axis tick with as many decimals as the tick step has, so that every tick of an axis
/// carries the same number of digits: `0.0, 0.5, 1.0` rather than `0, 0.5, 1`.
pub(crate) fn format_tick(value: f64, step: f64, style: NumberStyle) -> String {
    let step = match style.format {
        ValueFormat::Number => step,
        ValueFormat::Percent => step * 100.0,
    };
    let decimals = format!("{}", tidy(step))
        .split_once('.')
        .map_or(0, |(_, fraction)| fraction.len());
    format_value(
        value,
        NumberStyle {
            decimals: Some(u8::try_from(decimals).unwrap_or(u8::MAX)),
            ..style
        },
    )
}

/// Removes binary floating-point noise (`0.07 * 100`, `0.1 + 0.2`) by keeping 12 significant
/// digits, and normalizes `-0.0`. Rust's float formatting does not depend on the platform, so
/// the result stays deterministic.
fn tidy(value: f64) -> f64 {
    let rounded: f64 = format!("{value:.11e}")
        .parse()
        .expect("a formatted float parses back");
    if rounded == 0.0 { 0.0 } else { rounded }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct NumericScale {
    min: f64,
    max: f64,
    pub step: f64,
}

impl NumericScale {
    /// A scale over the values that also reaches the declared `(min, max)` of the value axis.
    /// Without zero in the scale, the values get a margin, which neither crosses zero nor goes
    /// beyond a declared bound; every end is then rounded outward to a tick.
    pub(crate) fn from_values(
        mut values: impl Iterator<Item = f64>,
        include_zero: bool,
        (declared_min, declared_max): (Option<f64>, Option<f64>),
    ) -> Self {
        let first = values.next().expect("validated charts contain values");
        let mut min = if include_zero { first.min(0.0) } else { first };
        let mut max = if include_zero { first.max(0.0) } else { first };
        for value in values {
            min = min.min(value);
            max = max.max(value);
        }
        min = declared_min.map_or(min, |bound| min.min(bound));
        max = declared_max.map_or(max, |bound| max.max(bound));
        if (max - min).abs() < f64::EPSILON {
            if max.abs() < f64::EPSILON {
                min = -1.0;
                max = 1.0;
            } else {
                let padding = max.abs() * 0.2;
                min -= padding;
                max += padding;
            }
        } else if !include_zero {
            let padding = (max - min) * 0.05;
            if declared_min.is_none() {
                min = if min >= 0.0 {
                    (min - padding).max(0.0)
                } else {
                    min - padding
                };
            }
            if declared_max.is_none() {
                max = if max <= 0.0 {
                    (max + padding).min(0.0)
                } else {
                    max + padding
                };
            }
        }
        let raw_step = (max - min) / 5.0;
        let magnitude = 10.0_f64.powf(raw_step.log10().floor());
        let fraction = raw_step / magnitude;
        let nice_fraction = if fraction <= 1.0 {
            1.0
        } else if fraction <= 2.0 {
            2.0
        } else if fraction <= 5.0 {
            5.0
        } else {
            10.0
        };
        let step = nice_fraction * magnitude;
        Self {
            min: tidy(tidy(min / step).floor() * step),
            max: tidy(tidy(max / step).ceil() * step),
            step,
        }
    }

    pub(crate) fn map(self, value: f64, output_min: f64, output_max: f64) -> f64 {
        let ratio = (value - self.min) / (self.max - self.min);
        output_min + ratio * (output_max - output_min)
    }

    /// Ticks are computed from their index instead of by repeated addition, so rounding errors
    /// do not accumulate along the axis.
    pub(crate) fn ticks(self) -> impl Iterator<Item = f64> {
        std::iter::successors(Some(0.0_f64), |index| Some(index + 1.0))
            .map(move |index| tidy(self.min + index * self.step))
            .take_while(move |tick| *tick <= self.max + self.step / 2.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{NumericScale, format_tick, format_value};
    use crate::spec::{Locale, NumberStyle, ValueFormat};

    #[test]
    fn scale_includes_zero_and_uses_nice_ticks() {
        let scale = NumericScale::from_values([12.0, 18.0, 15.0].into_iter(), true, (None, None));
        assert!(scale.min.abs() < f64::EPSILON);
        assert!((scale.max - 20.0).abs() < f64::EPSILON);
        assert_eq!(
            scale.ticks().collect::<Vec<_>>(),
            vec![0.0, 5.0, 10.0, 15.0, 20.0]
        );
    }

    #[test]
    fn a_margin_never_crosses_zero() {
        let scale = NumericScale::from_values([0.0, 40.0, 100.0].into_iter(), false, (None, None));
        assert!(scale.min.abs() < f64::EPSILON);
        let scale = NumericScale::from_values([-100.0, -3.0].into_iter(), false, (None, None));
        assert!(scale.max.abs() < f64::EPSILON);
        let scale = NumericScale::from_values([-10.0, 90.0].into_iter(), false, (None, None));
        assert!(scale.min < -10.0 && scale.max > 90.0);
    }

    #[test]
    fn a_declared_range_widens_the_scale_without_a_margin() {
        let scale =
            NumericScale::from_values([12.0, 48.0].into_iter(), false, (Some(10.0), Some(60.0)));
        assert!((scale.min - 10.0).abs() < f64::EPSILON);
        assert!((scale.max - 60.0).abs() < f64::EPSILON);
        let scale = NumericScale::from_values([12.0, 48.0].into_iter(), true, (None, Some(100.0)));
        assert_eq!(
            scale.ticks().collect::<Vec<_>>(),
            vec![0.0, 20.0, 40.0, 60.0, 80.0, 100.0]
        );
    }

    #[test]
    fn formats_percent_without_duplicate_suffix() {
        assert_eq!(format_value(0.125, ValueFormat::Percent), "12.5%");
        assert_eq!(format_value(1.0, ValueFormat::Percent), "100%");
    }

    #[test]
    fn removes_floating_point_noise_from_labels() {
        assert_eq!(format_value(0.07, ValueFormat::Percent), "7%");
        assert_eq!(format_value(0.1 + 0.2, ValueFormat::Number), "0.3");
        assert_eq!(format_value(-0.0, ValueFormat::Number), "0");
    }

    #[test]
    fn writes_german_numbers_with_comma_and_true_minus() {
        let de = NumberStyle {
            locale: Locale::De,
            ..NumberStyle::default()
        };
        assert_eq!(format_value(-1.25, de), "\u{2212}1,25");
        assert_eq!(format_value(-1.25, ValueFormat::Number), "\u{2212}1.25");
        let fixed = NumberStyle {
            decimals: Some(2),
            ..de
        };
        assert_eq!(format_value(1.547, fixed), "1,55");
        assert_eq!(format_value(-0.001, fixed), "0,00");
        let percent = NumberStyle {
            format: ValueFormat::Percent,
            ..de
        };
        assert_eq!(format_value(0.125, percent), "12,5\u{202f}%");
    }

    #[test]
    fn ticks_share_the_decimals_of_their_step() {
        let style = NumberStyle::default();
        assert_eq!(format_tick(1.0, 0.5, style), "1.0");
        assert_eq!(format_tick(-0.5, 0.5, style), "\u{2212}0.5");
        assert_eq!(format_tick(20.0, 5.0, style), "20");
        let percent = NumberStyle::from(ValueFormat::Percent);
        assert_eq!(format_tick(0.1, 0.05, percent), "10%");
        assert_eq!(format_tick(0.1, 0.025, percent), "10.0%");
    }

    #[test]
    fn ticks_do_not_accumulate_rounding_errors() {
        let scale = NumericScale::from_values([0.12, -0.04, 0.15].into_iter(), true, (None, None));
        assert_eq!(
            scale.ticks().collect::<Vec<_>>(),
            vec![-0.05, 0.0, 0.05, 0.1, 0.15]
        );
        let scale = NumericScale::from_values([0.07, -0.005, 0.3].into_iter(), true, (None, None));
        assert_eq!(
            scale
                .ticks()
                .map(|tick| format_value(tick, ValueFormat::Percent))
                .collect::<Vec<_>>(),
            vec!["−10%", "0%", "10%", "20%", "30%"]
        );
    }
}
