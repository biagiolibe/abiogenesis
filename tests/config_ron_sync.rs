// Guards against exactly the drift caught mid-session on `river_top_fraction`
// (0.006 in the RON, 0.004 in `SimConfig::default()`, undetected by
// `cargo test` since nothing ever deserializes the RON in the test suite).
// `assets/config/sim_config.ron`'s own header comment says the two must be
// kept in sync by hand — this test makes "by hand" actually enforced.

use abiogenesis::config::SimConfig;

#[test]
fn sim_config_ron_matches_defaults() {
    let ron_str = std::fs::read_to_string("assets/config/sim_config.ron")
        .expect("assets/config/sim_config.ron should exist and be readable");
    let from_ron: SimConfig = ron::from_str(&ron_str)
        .expect("assets/config/sim_config.ron should deserialize into SimConfig");
    let defaults = SimConfig::default();

    // `SimConfig` doesn't derive `PartialEq` (would require every nested
    // config struct to as well, for no other purpose) — comparing the
    // `Debug` representations is a deliberate, lighter-weight substitute:
    // deterministic field-by-field text, and a mismatch's assertion output
    // is a readable diff of exactly what drifted.
    assert_eq!(
        format!("{from_ron:#?}"),
        format!("{defaults:#?}"),
        "assets/config/sim_config.ron has drifted from SimConfig::default() — its own header \
         comment says the two must be kept in sync by hand; a config change likely updated \
         one without the other"
    );
}
