use crate::experiment006::Credit;
use serde::{Deserialize, Serialize};
pub const UNITS: u64 = 1_000_000;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_point_losses_match_independent_numeric_reference() {
        for numerator in 0..=1000 {
            let p = Probability::new(numerator, 1000).unwrap();
            for correct in [false, true] {
                let probability = numerator as f64 / 1000.0;
                let outcome = f64::from(correct);
                let brier = ((probability - outcome).powi(2) * UNITS as f64).round() as u64;
                let likelihood = if correct {
                    probability
                } else {
                    1.0 - probability
                };
                let loss = (-likelihood.clamp(1.0 / 1024.0, 1023.0 / 1024.0).ln() * UNITS as f64)
                    .floor() as u64;
                assert!(p.brier(correct).abs_diff(brier) <= 1);
                assert!(p.log_loss(correct).abs_diff(loss) <= 1);
            }
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Probability {
    pub numerator: u64,
    pub denominator: u64,
}
impl Probability {
    pub fn new(numerator: u64, denominator: u64) -> Result<Self, String> {
        if denominator == 0 || numerator > denominator || denominator > 1_000_000 {
            return Err("invalid rational probability".into());
        }
        Ok(Self {
            numerator,
            denominator,
        })
    }
    pub fn ppm(self) -> u64 {
        ((u128::from(self.numerator) * u128::from(UNITS) + u128::from(self.denominator) / 2)
            / u128::from(self.denominator)) as u64
    }
    pub fn brier(self, correct: bool) -> u64 {
        let error = if correct {
            self.denominator - self.numerator
        } else {
            self.numerator
        };
        let d = u128::from(self.denominator).pow(2);
        ((u128::from(error).pow(2) * u128::from(UNITS) + d / 2) / d) as u64
    }
    pub fn log_loss(self, correct: bool) -> u64 {
        const SCALE: i128 = 1_000_000_000_000_000;
        let n = if correct {
            self.numerator
        } else {
            self.denominator - self.numerator
        };
        let mut x = (i128::from(n) * SCALE / i128::from(self.denominator))
            .clamp(SCALE / 1024, SCALE * 1023 / 1024);
        let mut steps = 0;
        while x < SCALE / 2 {
            x *= 2;
            steps += 1;
        }
        let z = (x - SCALE) * SCALE / (x + SCALE);
        let square = z * z / SCALE;
        let mut term = z;
        let mut sum = 0;
        for k in 0..32 {
            sum += term / (2 * k + 1);
            term = term * square / SCALE;
        }
        ((i128::from(steps) * 693_147_180_559_945 - 2 * sum) / 1_000_000_000) as u64
    }
    pub fn bin(self) -> usize {
        (self.numerator * 5 / self.denominator).min(4) as usize
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredictionInput {
    pub source: u8,
    pub score: i8,
    pub accurate: u16,
    pub inaccurate: u16,
}
impl PredictionInput {
    pub fn from_credit(source: u8, c: Credit) -> Self {
        Self {
            source,
            score: c.score,
            accurate: c.accurate,
            inaccurate: c.inaccurate,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Forecast {
    pub input: PredictionInput,
    pub neutral: Probability,
    pub frequency: Probability,
    pub ordinal: Option<bool>,
}
pub trait Predictor {
    fn predict(&mut self, input: PredictionInput) -> Forecast;
}
pub struct FrequencyPredictor;
impl Predictor for FrequencyPredictor {
    fn predict(&mut self, input: PredictionInput) -> Forecast {
        Forecast {
            input,
            neutral: Probability {
                numerator: 1,
                denominator: 2,
            },
            frequency: Probability {
                numerator: u64::from(input.accurate) + 1,
                denominator: u64::from(input.accurate) + u64::from(input.inaccurate) + 2,
            },
            ordinal: match input.score.cmp(&0) {
                std::cmp::Ordering::Greater => Some(true),
                std::cmp::Ordering::Less => Some(false),
                _ => None,
            },
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bin {
    pub samples: u64,
    pub forecast_sum_ppm: u64,
    pub correct: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Calibration {
    pub samples: u64,
    pub brier_sum: u64,
    pub log_loss_sum: u64,
    pub bins: [Bin; 5],
}
impl Calibration {
    pub fn add(&mut self, p: Probability, correct: bool) {
        self.samples += 1;
        self.brier_sum += p.brier(correct);
        self.log_loss_sum += p.log_loss(correct);
        let bin = &mut self.bins[p.bin()];
        bin.samples += 1;
        bin.forecast_sum_ppm += p.ppm();
        bin.correct += u64::from(correct);
    }
    pub fn brier(&self) -> Option<u64> {
        ratio(self.brier_sum, self.samples)
    }
    pub fn log_loss(&self) -> Option<u64> {
        ratio(self.log_loss_sum, self.samples)
    }
    pub fn ece(&self) -> Option<u64> {
        ratio(
            self.bins
                .iter()
                .map(|b| b.forecast_sum_ppm.abs_diff(b.correct * UNITS))
                .sum(),
            self.samples,
        )
    }
}
pub fn ratio(n: u64, d: u64) -> Option<u64> {
    n.checked_div(d)
}
