#![allow(dead_code)]

use self::Value::{Extrapolated, Interpolated};

#[derive(PartialEq, Debug)]
pub enum Value {
    Interpolated(f64),
    Extrapolated(f64),
    NoValue,
}

impl Value {
    pub fn is_interp(&self) -> bool {
        matches!(*self, Interpolated(_))
    }

    pub fn is_extrap(&self) -> bool {
        matches!(*self, Extrapolated(_))
    }

    pub fn is_value(&self) -> bool {
        matches!(*self, Interpolated(_) | Extrapolated(_))
    }

    pub fn to_interp(&self) -> Option<f64> {
        match *self {
            Interpolated(v) => Some(v),
            _ => None,
        }
    }

    pub fn to_extrap(&self) -> Option<f64> {
        match *self {
            Extrapolated(v) => Some(v),
            _ => None,
        }
    }

    pub fn to_value(&self) -> Option<f64> {
        match *self {
            Interpolated(v) | Extrapolated(v) => Some(v),
            _ => None,
        }
    }
}

pub(crate) fn interpolate(x: f64, xs: &[f64], ys: &[f64]) -> Value {
    use self::Value::NoValue;

    if xs.is_empty() {
        return NoValue;
    }

    match xs.binary_search_by(|v| v.partial_cmp(&x).expect("error in binary search")) {
        Ok(i) => {
            let y = ys.get(i).expect("error getting y0 in interpolator");

            Interpolated(*y)
        }
        Err(i) => {
            if i == 0 {
                let x0 = xs.get(i).expect("error getting x0 in interpolator");
                let y0 = ys.get(i).expect("error getting y0 in interpolator");
                if let (Some(x1), Some(y1)) = (xs.get(i + 1), ys.get(i + 1)) {
                    Extrapolated((y1 - y0) / (x1 - x0) * (x - x0) + y0)
                } else {
                    Extrapolated(*y0)
                }
            } else if i == xs.len() {
                let x1 = xs.get(i - 1).expect("error getting x1 in interpolator");
                let y1 = ys.get(i - 1).expect("error getting y1 in interpolator");
                if let (Some(x0), Some(y0)) = (xs.get(i - 2), ys.get(i - 2)) {
                    Extrapolated((y1 - y0) / (x1 - x0) * (x - x1) + y1)
                } else {
                    Extrapolated(*y1)
                }
            } else {
                let x0 = xs.get(i - 1).expect("error getting x0 in interpolator");
                let y0 = ys.get(i - 1).expect("error getting y0 in interpolator");
                let x1 = xs.get(i).expect("error getting x1 in interpolator");
                let y1 = ys.get(i).expect("error getting y1 in interpolator");

                Interpolated((y1 - y0) / (x1 - x0) * (x - x0) + y0)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolation() {
        use super::Value::NoValue;
        use std::f64::EPSILON;

        let x = interpolate(0.0, &[], &[]);
        assert!(!x.is_interp());
        assert!(!x.is_extrap());
        assert!(!x.is_value());
        assert_eq!(x, NoValue);

        let x = interpolate(0.0, &[0.0], &[1.0]);
        assert!(x.is_interp());
        assert!(!x.is_extrap());
        assert!(x.is_value());
        assert!(f64::abs(x.to_interp().unwrap() - 1.0) < EPSILON);
        assert!(x.to_extrap().is_none());
        assert!(f64::abs(x.to_value().unwrap() - 1.0) < EPSILON);

        let x = interpolate(10.0, &[100.0, 200.0], &[3.0, 4.0]);
        assert!(!x.is_interp());
        assert!(x.is_extrap());
        assert!(x.is_value());
        assert!(x.to_interp().is_none());
        assert!(f64::abs(x.to_extrap().unwrap() - 2.1) < EPSILON);
        assert!(f64::abs(x.to_value().unwrap() - 2.1) < EPSILON);

        let x = interpolate(210.0, &[100.0, 200.0], &[3.0, 4.0]);
        assert!(!x.is_interp());
        assert!(x.is_extrap());
        assert!(x.is_value());
        assert!(x.to_interp().is_none());
        assert!(f64::abs(x.to_extrap().unwrap() - 4.1) < EPSILON);
        assert!(f64::abs(x.to_value().unwrap() - 4.1) < EPSILON);

        let x = interpolate(100.0, &[100.0, 200.0], &[3.0, 4.0]);
        assert!(x.is_interp());
        assert!(!x.is_extrap());
        assert!(x.is_value());
        assert!(f64::abs(x.to_interp().unwrap() - 3.0) < EPSILON);
        assert!(x.to_extrap().is_none());
        assert!(f64::abs(x.to_value().unwrap() - 3.0) < EPSILON);

        let x = interpolate(200.0, &[100.0, 200.0], &[3.0, 4.0]);
        assert!(x.is_interp());
        assert!(!x.is_extrap());
        assert!(x.is_value());
        assert!(f64::abs(x.to_interp().unwrap() - 4.0) < EPSILON);
        assert!(x.to_extrap().is_none());
        assert!(f64::abs(x.to_value().unwrap() - 4.0) < EPSILON);
    }
}
