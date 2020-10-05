use crate::{
    kinematics::KinematicLine,
    reaction::Type::{self, Ar34, Cl34, S34},
};
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub static ref DATA: Data = {
        let mut kinematic_lines = HashMap::new();
        kinematic_lines.insert(S34, get_kin_lines(include_str!("data/34S.txt")));
        kinematic_lines.insert(Cl34, get_kin_lines(include_str!("data/34Cl.txt")));
        kinematic_lines.insert(Ar34, get_kin_lines(include_str!("data/34Ar.txt")));
        let mut xsecs = HashMap::new();
        xsecs.insert(S34, get_xsecs(include_str!("data/34S_a_p__55.44MeV.out")));
        xsecs.insert(Cl34, get_xsecs(include_str!("data/34Cl_a_p__55.44MeV.out")));
        xsecs.insert(Ar34, get_xsecs(include_str!("data/34Ar_a_p__55.44MeV.out")));
        let angles = get_angles(include_str!("data/det_angles.cfg"));

        Data {
            kinematic_lines,
            xsecs,
            angles,
        }
    };
}

pub struct Data {
    pub kinematic_lines: HashMap<Type, Vec<KinematicLine>>,
    pub xsecs: HashMap<Type, Vec<f64>>,
    pub angles: Vec<AngleInfo>,
}

#[derive(Debug, Clone)]
pub struct AngleInfo {
    pub theta_min: f64,
    pub theta_max: f64,
    pub theta_avg: f64,
    pub phi_min: f64,
    pub phi_max: f64,
    pub phi_avg: f64,
}

fn get_kin_lines(s: &str) -> Vec<KinematicLine> {
    let mut kin_lines = Vec::new();

    let mut com_angles = Vec::new();
    let mut ejectile_angles = Vec::new();
    let mut ejectile_energies = Vec::new();
    let mut recoil_angles = Vec::new();
    let mut recoil_energies = Vec::new();
    let mut weights = Vec::new();

    for l in s.lines() {
        let x: Vec<_> = l.split_whitespace().collect();

        if x.is_empty() && !com_angles.is_empty() {
            // If there's an empty line, we're starting a new kinematic line
            // Don't add a new kinematic line if it's empty
            kin_lines.push(
                KinematicLine::new(
                    com_angles,
                    ejectile_angles,
                    ejectile_energies,
                    recoil_angles,
                    recoil_energies,
                    weights,
                )
                .unwrap(),
            );
            com_angles = Vec::new();
            ejectile_angles = Vec::new();
            ejectile_energies = Vec::new();
            recoil_angles = Vec::new();
            recoil_energies = Vec::new();
            weights = Vec::new();
        } else if x[0].starts_with('#') {
            // Ignore comments
            continue;
        } else {
            // Build current kinematic line
            com_angles.push(x[0].parse::<f64>().unwrap());
            ejectile_angles.push(x[1].parse::<f64>().unwrap());
            ejectile_energies.push(x[2].parse::<f64>().unwrap());
            recoil_angles.push(x[4].parse::<f64>().unwrap());
            recoil_energies.push(x[5].parse::<f64>().unwrap());
            weights.push(1.0);
        }
    }

    // If there's any left over data, add it as another kinematic line
    // This is important if the file doesn't end with a new line
    if !com_angles.is_empty() {
        kin_lines.push(
            KinematicLine::new(
                com_angles,
                ejectile_angles,
                ejectile_energies,
                recoil_angles,
                recoil_energies,
                weights,
            )
            .unwrap(),
        );
    }

    kin_lines
}

fn get_xsecs(s: &str) -> Vec<f64> {
    let mut xsecs = Vec::new();

    for l in s.lines() {
        let x: Vec<_> = l.split_whitespace().collect();
        if x.is_empty() || x[0].starts_with('#') {
            continue;
        }
        let xsec = x[6].parse::<f64>().unwrap();
        xsecs.push(xsec);
    }

    xsecs
}

fn get_angles(s: &str) -> Vec<AngleInfo> {
    let mut angles = Vec::new();

    for l in s.lines() {
        let x: Vec<_> = l.split_whitespace().collect();
        if x.is_empty() || x[0].starts_with('#') {
            continue;
        }
        if x.len() < 8 {
            eprintln!("WARNING: Error parsing a line in the angle file.");
        } else {
            let theta_min = x[2].parse::<f64>().unwrap();
            let theta_max = x[3].parse::<f64>().unwrap();
            let theta_avg = x[4].parse::<f64>().unwrap();

            let phi_min = x[5].parse::<f64>().unwrap();
            let phi_max = x[6].parse::<f64>().unwrap();
            let phi_avg = x[7].parse::<f64>().unwrap();

            angles.push(AngleInfo {
                theta_min,
                theta_max,
                theta_avg,
                phi_min,
                phi_max,
                phi_avg,
            });
        }
    }

    angles
}
