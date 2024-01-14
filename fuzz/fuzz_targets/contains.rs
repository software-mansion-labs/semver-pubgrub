#![no_main]
use std::ops::RangeBounds;

use libfuzzer_sys::fuzz_target;
use proptest::strategy::Strategy;
use proptest::test_runner::TestError;
use proptest::test_runner::{Config, RngAlgorithm, TestRng, TestRunner};
use pubgrub::version_set::VersionSet;
use semver_pubgrub::SemverPubgrub;
use semver_pubgrub_fuzz::{req_strategy, version_strategy};

// cargo fuzz run contains

fn contains(req: &semver::VersionReq, ver: &semver::Version) {
    // println!("{req} |=> {ver}");
    let pver: SemverPubgrub = req.into();
    let neg = pver.complement();
    let mat = req.matches(&ver);
    assert_eq!(mat, pver.contains(&ver), "matches {req} |=> {ver}");
    assert_eq!(!mat, neg.contains(&ver), "!matches {req} |=> {ver}");

    let bounding_range = pver.bounding_range();
    if bounding_range.is_some_and(|b| !b.contains(&ver)) {
        assert!(!mat);
    }
    if mat {
        assert!(bounding_range.unwrap().contains(&ver));
    }
}

fn case(seed: &[u8]) {
    let mut test_runner = TestRunner::new_with_rng(
        Config {
            cases: 1,
            failure_persistence: None,
            ..Config::default()
        },
        TestRng::from_seed(RngAlgorithm::PassThrough, seed),
    );
    let strategy = &(req_strategy(), version_strategy());
    let new_tree = strategy.new_tree(&mut test_runner).unwrap();
    let result = test_runner.run_one(new_tree, |v| {
        contains(&v.0, &v.1);
        Ok(())
    });

    if let Err(TestError::Fail(_, (req, ver))) = result {
        panic!("Found minimal failing case: {req} |=> {ver}");
    }
}

fuzz_target!(|seed: &[u8]| case(seed));
