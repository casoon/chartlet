use serde::{Deserialize, Serialize};

use super::{ChartSpec, default_true, validate_number};
use crate::error::{ChartError, ChartWarning};

/// Warming stripes: one stripe per year; beyond this the stripes get thinner than a pixel at the
/// default width.
pub(crate) const MAX_STRIPES: usize = 500;

/// A diverging color scale around a reference value, shared by stripes and calendars. Values are
/// sorted into eight steps on either side of the reference; `min` and `max` set where the outermost
/// step begins, and default to the largest distance from the reference on either side.
pub(crate) struct Diverging {
    pub reference: f64,
    pub min: f64,
    pub max: f64,
}

/// Warming stripes: one colored stripe per year.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StripesSpec {
    /// The year of the first value; every further value is the following year.
    pub first_year: i32,
    /// One value per year; `null` leaves the year empty.
    pub values: Vec<Option<f64>>,
    /// The value the scale diverges from, drawn in the neutral middle color.
    #[serde(default)]
    pub reference: f64,
    /// Where the coldest step begins; defaults to the reference minus the largest distance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    /// Where the warmest step begins; defaults to the reference plus the largest distance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// Label the first and the last year under the stripes.
    #[serde(default = "default_true")]
    pub year_labels: bool,
}

impl StripesSpec {
    /// The year of every value, in order.
    pub(crate) fn years(&self) -> impl Iterator<Item = i32> + '_ {
        (0..self.values.len()).map(|offset| {
            self.first_year + i32::try_from(offset).expect("stripes are limited to 500 values")
        })
    }

    pub(crate) fn diverging(&self) -> Diverging {
        Diverging::new(
            self.reference,
            self.min,
            self.max,
            self.values.iter().flatten().copied(),
        )
    }
}

impl Diverging {
    /// Steps on each side of the reference; with the middle this makes 17 colors.
    pub(crate) const STEPS: usize = 8;

    pub(super) fn new(
        reference: f64,
        min: Option<f64>,
        max: Option<f64>,
        values: impl Iterator<Item = f64>,
    ) -> Self {
        let reach = values
            .map(|value| (value - reference).abs())
            .fold(0.0, f64::max);
        // All values on the reference: any positive reach draws them in the middle color.
        let reach = if reach > 0.0 { reach } else { 1.0 };
        Self {
            reference,
            min: min.unwrap_or(reference - reach),
            max: max.unwrap_or(reference + reach),
        }
    }

    /// The color step of a value, from 0 (coldest) over [`Self::STEPS`] (the reference) to
    /// `2 × STEPS` (warmest). Each step is equally wide; values beyond `min` or `max` take the
    /// outermost step.
    pub(crate) fn step(&self, value: f64) -> usize {
        let (distance, reach) = if value >= self.reference {
            (value - self.reference, self.max - self.reference)
        } else {
            (self.reference - value, self.reference - self.min)
        };
        // `STEPS` is 8, so the product stays tiny and the cast is exact.
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let arm = ((distance / reach * Self::STEPS as f64).round() as usize).min(Self::STEPS);
        if value >= self.reference {
            Self::STEPS + arm
        } else {
            Self::STEPS - arm
        }
    }
}

impl ChartSpec {
    /// Warming stripes: a block of consecutive yearly values and a diverging scale.
    pub(super) fn validate_stripes(&self) -> Result<Vec<ChartWarning>, ChartError> {
        self.reject_map_options("a stripes chart", "stripes.values")?;
        let Some(stripes) = &self.stripes else {
            return Err(ChartError::new(
                "missing_stripes",
                "/stripes",
                "a stripes chart requires a stripes block",
            ));
        };
        if stripes.values.is_empty() {
            return Err(ChartError::new(
                "empty_data",
                "/stripes/values",
                "provide at least one value",
            ));
        }
        if stripes.values.len() > MAX_STRIPES {
            return Err(ChartError::new(
                "too_many_data_points",
                "/stripes/values",
                format!("at most {MAX_STRIPES} yearly values are supported"),
            ));
        }
        let last_year = i64::from(stripes.first_year) + count_i64(stripes.values.len()) - 1;
        if stripes.first_year < 1 || last_year > 9_999 {
            return Err(ChartError::new(
                "invalid_year",
                "/stripes/firstYear",
                "the years must lie between 1 and 9999",
            ));
        }
        for (index, value) in stripes.values.iter().enumerate() {
            if let Some(value) = value {
                validate_number(*value, &format!("/stripes/values/{index}"))?;
            }
        }
        if stripes.values.iter().all(Option::is_none) {
            return Err(ChartError::new(
                "empty_series",
                "/stripes/values",
                "provide at least one numeric value",
            ));
        }
        validate_diverging(stripes.reference, stripes.min, stripes.max, "/stripes")?;
        Ok(Vec::new())
    }
}

/// The reference and the optional ends of a diverging scale: `min` below the reference and `max`
/// above it, so that both arms of the scale have a direction.
pub(super) fn validate_diverging(
    reference: f64,
    min: Option<f64>,
    max: Option<f64>,
    path: &str,
) -> Result<(), ChartError> {
    validate_number(reference, &format!("{path}/reference"))?;
    if let Some(min) = min {
        validate_number(min, &format!("{path}/min"))?;
        if min >= reference {
            return Err(ChartError::new(
                "invalid_scale",
                format!("{path}/min"),
                "min must lie below reference",
            ));
        }
    }
    if let Some(max) = max {
        validate_number(max, &format!("{path}/max"))?;
        if max <= reference {
            return Err(ChartError::new(
                "invalid_scale",
                format!("{path}/max"),
                "max must lie above reference",
            ));
        }
    }
    Ok(())
}

/// A count as a signed number, for year arithmetic.
fn count_i64(value: usize) -> i64 {
    i64::try_from(value).expect("counts are limited by validation")
}
