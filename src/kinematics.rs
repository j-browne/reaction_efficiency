#![allow(dead_code)]

use crate::interpolation::{interpolate, Value};
use std::fmt;

pub struct KinematicLine {
    com_thetas: Vec<f64>,
    ejectile_thetas: Vec<f64>,
    ejectile_energies: Vec<f64>,
    recoil_thetas: Vec<f64>,
    recoil_energies: Vec<f64>,
    weights: Vec<f64>,
}

impl KinematicLine {
    pub fn new(
        com_thetas: Vec<f64>,
        ejectile_thetas: Vec<f64>,
        ejectile_energies: Vec<f64>,
        recoil_thetas: Vec<f64>,
        recoil_energies: Vec<f64>,
        weights: Vec<f64>,
    ) -> Option<KinematicLine> {
        // Verify that com_angles is sorted and unique
        let mut last = ::std::f64::NEG_INFINITY;
        for a in &com_thetas {
            if *a <= last {
                return None;
            }
            last = *a;
        }

        // Verify that all Vecs are the same size
        if com_thetas.len() != ejectile_thetas.len()
            || com_thetas.len() != ejectile_energies.len()
            || com_thetas.len() != recoil_thetas.len()
            || com_thetas.len() != recoil_energies.len()
            || com_thetas.len() != weights.len()
        {
            return None;
        }
        Some(KinematicLine {
            com_thetas,
            ejectile_thetas,
            ejectile_energies,
            recoil_thetas,
            recoil_energies,
            weights,
        })
    }

    pub fn ejectile_theta(&self, com_theta: f64) -> Value {
        interpolate(com_theta, &self.com_thetas, &self.ejectile_thetas)
    }

    pub fn ejectile_energy(&self, com_theta: f64) -> Value {
        interpolate(com_theta, &self.com_thetas, &self.ejectile_energies)
    }

    pub fn recoil_theta(&self, com_theta: f64) -> Value {
        interpolate(com_theta, &self.com_thetas, &self.recoil_thetas)
    }

    pub fn recoil_energy(&self, com_theta: f64) -> Value {
        interpolate(com_theta, &self.com_thetas, &self.recoil_energies)
    }

    pub fn weight(&self, com_theta: f64) -> Value {
        interpolate(com_theta, &self.com_thetas, &self.weights)
    }
}

impl fmt::Debug for KinematicLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KinematicLine").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kin_line_vec_sizes() {
        assert!(KinematicLine::new(vec![], vec![], vec![], vec![], vec![], vec![]).is_some());
        assert!(KinematicLine::new(vec![0.0], vec![], vec![], vec![], vec![], vec![]).is_none());
        assert!(KinematicLine::new(
            vec![0.0],
            vec![0.0],
            vec![0.0],
            vec![0.0],
            vec![0.0],
            vec![0.0],
        )
        .is_some());
    }

    #[test]
    fn kin_line_invalid() {
        assert!(KinematicLine::new(
            vec![0.0, 1.0],
            vec![1.0, 1.0],
            vec![1.0, 1.0],
            vec![1.0, 1.0],
            vec![1.0, 1.0],
            vec![1.0, 1.0],
        )
        .is_some());
        assert!(KinematicLine::new(
            vec![0.0, 0.0],
            vec![1.0, 1.0],
            vec![1.0, 1.0],
            vec![1.0, 1.0],
            vec![1.0, 1.0],
            vec![1.0, 1.0],
        )
        .is_none());
        assert!(KinematicLine::new(
            vec![0.0, 1.0, 0.5],
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
        )
        .is_none());
    }
}
