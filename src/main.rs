#![feature(bool_to_option)]

#[macro_use]
extern crate serde_derive;

use crate::{data::DATA, reaction::Reaction};
use anyhow::Error;
use rand::{
    distributions::{weighted::WeightedIndex, Distribution, Uniform},
    thread_rng,
};
use rayon::{iter::repeat, prelude::*};
use std::{fs::File, io::BufWriter, path::PathBuf, sync::Mutex};
use structopt::{
    clap::{AppSettings, ArgGroup},
    StructOpt,
};

mod data;
mod interpolation;
mod kinematics;
mod reaction;

/// Monte Carlo simulation to calculate detector efficiency.
#[derive(Debug, StructOpt)]
#[structopt(
    no_version,
    group = ArgGroup::with_name("action").required(true).multiple(false),
    group = ArgGroup::with_name("species").required(true).multiple(true),
    setting = AppSettings::DeriveDisplayOrder
)]
struct Opt {
    /// Simulate each state separately with <iter> iterations each
    #[structopt(long, name = "iter-state", group = "action")]
    per_state: Option<u32>,
    /// Simulate each reaction separately with <iter> iterations each, with states for each
    /// reaction chosen weighted on their Hauser-Feshbach cross section prediction
    #[structopt(long, name = "iter-reaction", group = "action")]
    per_reaction: Option<u32>,
    /// Run simulations for 34S
    #[structopt(long, group = "species")]
    s34: bool,
    /// Run simulations for 34Cl
    #[structopt(long, group = "species")]
    cl34: bool,
    /// Run simulations for 34Ar
    #[structopt(long, group = "species")]
    ar34: bool,
    /// Hits output file
    #[structopt(long, parse(from_os_str))]
    hits: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Hit {
    pub ejectile_theta: f64,
    pub ejectile_phi: f64,
    pub ejectile_energy: f64,
    pub recoil_theta: f64,
    pub recoil_phi: f64,
    pub recoil_energy: f64,
}

fn main() -> Result<(), Error> {
    use reaction::{
        Level::{ExcitedState, Overall},
        Type::{Ar34, Cl34, S34},
    };

    let opt = Opt::from_args();
    let out_effs = Mutex::new(Vec::new());
    let out_hits = opt.hits.as_ref().and(Some(Mutex::new(Vec::new())));

    // Get kinematic lines
    let reactions = [
        opt.s34
            .then(|| (S34, &DATA.kinematic_lines[&S34], &DATA.xsecs[&S34])),
        opt.cl34
            .then(|| (Cl34, &DATA.kinematic_lines[&Cl34], &DATA.xsecs[&Cl34])),
        opt.ar34
            .then(|| (Ar34, &DATA.kinematic_lines[&Ar34], &DATA.xsecs[&Ar34])),
    ];

    // Either:
    // * iterate over each state `iter_states` times
    // * iterate over each reaction `iter_reactions` times, randomizing the states weighted by the
    //   cross sections provided
    // structopt should guarantee that one and only one of these options is present. There's
    // probably a more idiomatic way to do this (e.g. with subcommands), but this works.
    match (opt.per_state, opt.per_reaction) {
        (Some(iter_states), None) => {
            //let overall = Mutex::new(HashMap::new());

            reactions
                .par_iter()
                .flatten()
                .map(|(reaction_type, lines, xsecs)| {
                    assert_eq!(lines.len(), xsecs.len());
                    repeat(reaction_type).zip(lines.par_iter().zip(xsecs.par_iter()).enumerate())
                })
                .flatten()
                .for_each(|(reaction_type, (excited_state, (line, _xsec)))| {
                    let reaction = Reaction(*reaction_type, ExcitedState(excited_state as u64));

                    let mut hits = (0..iter_states)
                        .into_par_iter()
                        .map(|_| {
                            let rng = &mut thread_rng();

                            // TODO: Weight the angles based on something else?
                            // This weights it based solely on solid angle
                            // For a PDF(x) = sin(x) from 0 to pi,
                            // the CDF(x) = 0.5 * (1 - cos(x))
                            // Using the inverse of that, you can gen random numbers
                            let rand_num = Uniform::new(0.0, 1.0).sample(rng);
                            let com_theta = f64::acos(1.0 - 2.0 * rand_num).to_degrees();
                            let phi = Uniform::new(0.0, 360.0).sample(rng);

                            let hit = Hit {
                                ejectile_theta: line.ejectile_theta(com_theta).to_value().unwrap(),
                                ejectile_phi: phi,
                                ejectile_energy: line
                                    .ejectile_energy(com_theta)
                                    .to_value()
                                    .unwrap(),
                                recoil_theta: line.recoil_theta(com_theta).to_value().unwrap(),
                                recoil_phi: (phi + 180.0) % 360.0,
                                recoil_energy: line.recoil_energy(com_theta).to_value().unwrap(),
                            };
                            let detected = is_detected(&hit);
                            (reaction, hit, detected)
                        })
                        .collect::<Vec<_>>();

                    let num_detected = hits.iter().filter(|(_, _, detected)| *detected).count();
                    let eff = (num_detected as f64) / f64::from(iter_states);
                    let eff_err = (num_detected as f64).sqrt() / f64::from(iter_states);
                    out_effs
                        .lock()
                        .expect("could not lock `out` Mutex")
                        .push((reaction, eff, eff_err));

                    //let mut overall = overall.lock().expect("could not lock `overall` Mutex");
                    //let ov = overall.entry(reaction_type).or_insert((0.0, 0.0, 0.0));
                    //ov.0 += xsec * eff;
                    //if eff != 0.0 {
                    //    ov.1 += (xsec * eff_err).powi(2);
                    //}
                    //ov.2 += xsec;

                    if let Some(out_hits) = &out_hits {
                        out_hits
                            .lock()
                            .expect("could not lock `hits` Mutex")
                            .append(&mut hits);
                    }
                });

            //for (reaction_type, (sum, err_sum, weight_sum)) in overall.into_inner().expect("could not unwrap `overall` Mutex") {
            //    out.lock().expect("could not lock `out` Mutex").push(
            //        (Reaction(*reaction_type, Overall), sum / weight_sum, sum * err_sum.sqrt() / weight_sum)
            //    );
            //}
        }
        (None, Some(iter_reactions)) => {
            reactions
                .par_iter()
                .flatten()
                .for_each(|(reaction_type, lines, xsecs)| {
                    assert_eq!(lines.len(), xsecs.len());
                    let mut hits = (0..iter_reactions)
                        .into_par_iter()
                        .map(|_| {
                            let rng = &mut thread_rng();

                            // Choose an excited state, weighted by its xsec
                            let weights = WeightedIndex::new(xsecs.iter())
                                .expect("failed to make WeightedIndex");

                            let excited_state = weights.sample(rng);
                            let line = &lines[excited_state];
                            let reaction =
                                Reaction(*reaction_type, ExcitedState(excited_state as u64));

                            // TODO: Weight the angles based on something else?
                            // This weights it based solely on solid angle
                            // For a PDF(x) = sin(x) from 0 to pi,
                            // the CDF(x) = 0.5 * (1 - cos(x))
                            // Using the inverse of that, you can gen random numbers
                            let rand_num = Uniform::new(0.0, 1.0).sample(rng);
                            let com_theta = f64::acos(1.0 - 2.0 * rand_num).to_degrees();
                            let phi = Uniform::new(0.0, 360.0).sample(rng);

                            let hit = Hit {
                                ejectile_theta: line.ejectile_theta(com_theta).to_value().unwrap(),
                                ejectile_phi: phi,
                                ejectile_energy: line
                                    .ejectile_energy(com_theta)
                                    .to_value()
                                    .unwrap(),
                                recoil_theta: line.recoil_theta(com_theta).to_value().unwrap(),
                                recoil_phi: (phi + 180.0) % 360.0,
                                recoil_energy: line.recoil_energy(com_theta).to_value().unwrap(),
                            };
                            let detected = is_detected(&hit);
                            (reaction, hit, detected)
                        })
                        .collect::<Vec<_>>();

                    let num_detected = hits.iter().filter(|(_, _, detected)| *detected).count();
                    out_effs.lock().expect("could not lock `out` Mutex").push((
                        Reaction(*reaction_type, Overall),
                        (num_detected as f64) / f64::from(iter_reactions),
                        (num_detected as f64).sqrt() / f64::from(iter_reactions),
                    ));

                    if let Some(out_hits) = &out_hits {
                        out_hits
                            .lock()
                            .expect("could not lock `hits` Mutex")
                            .append(&mut hits);
                    }
                });
        }
        _ => unreachable!(),
    };

    // Output results
    println!(
        "{}",
        serde_json::to_string_pretty(&out_effs.into_inner().expect("could not unwrap `out` Mutex"))
            .expect("could not serialize `out`")
    );
    if let Some(filename) = opt.hits {
        let file = File::create(filename)?;
        let file = BufWriter::new(file);
        serde_json::to_writer_pretty(
            file,
            &out_hits
                .expect("`out_hits` was None when `opt.hits` was Some")
                .into_inner()
                .expect("could not unwrap `hits` Mutex"),
        )
        .expect("could not serialize `out`")
    }

    Ok(())
}

fn is_detected(h: &Hit) -> bool {
    if h.recoil_theta < 0.6 {
        return false;
    }
    if h.ejectile_energy < 0.5000 {
        return false;
    }
    for a in &DATA.angles {
        if (h.ejectile_theta > a.theta_min && h.ejectile_theta < a.theta_max)
            && (h.ejectile_phi > a.phi_min && h.ejectile_phi < a.phi_max)
        {
            return true;
        }
    }
    false
}
