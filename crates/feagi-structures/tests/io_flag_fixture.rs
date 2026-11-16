//! The shared IO-flag fixture (`io-flag-fixture.json`): the same table is asserted by Composer and
//! feagi-mcp, so the three decoders agree on what each flag defines and which flags are invalid.

use feagi_structures::genomic::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, IOCorticalAreaConfigurationFlag, PercentageNeuronPositioning,
};
use feagi_structures::genomic::cortical_area::CorticalID;
use serde_json::Value;

const FIXTURE: &str = include_str!("io-flag-fixture.json");

fn frame_name(frame: FrameChangeHandling) -> &'static str {
    match frame {
        FrameChangeHandling::Absolute => "Absolute",
        FrameChangeHandling::Incremental => "Incremental",
    }
}

fn positioning_name(positioning: PercentageNeuronPositioning) -> &'static str {
    match positioning {
        PercentageNeuronPositioning::Linear => "Linear",
        PercentageNeuronPositioning::Fractional => "Fractional",
    }
}

#[test]
fn shared_fixture_matches_the_flag_decoder() {
    let fixture: Value = serde_json::from_str(FIXTURE).expect("fixture parses");
    let cases = fixture["cases"].as_array().expect("cases");
    assert!(cases.len() >= 15);
    for case in cases {
        let flag = case["flag"].as_u64().expect("flag") as u16;
        let note = case["note"].as_str().unwrap_or("");
        let decoded = IOCorticalAreaConfigurationFlag::try_from_data_type_configuration_flag(flag);
        if case["expect"] == "invalid" {
            assert!(decoded.is_err(), "{note}: flag {flag:#06x} must be refused");
            continue;
        }
        let fields = decoded
            .unwrap_or_else(|e| panic!("{note}: flag {flag:#06x} refused: {e:?}"))
            .port_encoding_fields();
        let expect = &case["expect"];
        assert_eq!(expect["kind"], fields.kind, "{note}: kind");
        assert_eq!(
            expect.get("signed").and_then(Value::as_bool),
            fields.signed,
            "{note}: signed"
        );
        assert_eq!(
            expect.get("frame").and_then(Value::as_str),
            fields.frame.map(frame_name),
            "{note}: frame"
        );
        assert_eq!(
            expect.get("positioning").and_then(Value::as_str),
            fields.positioning.map(positioning_name),
            "{note}: positioning"
        );

        // The same flag read from a cortical id (IPU and OPU) gives the same result.
        for prefix in *b"io" {
            let [lo, hi] = flag.to_le_bytes();
            let bytes = [prefix, b'a', b'b', b'c', lo, hi, 0, 0];
            let id = CorticalID::try_from_bytes(&bytes).expect("io id");
            assert_eq!(
                id.extract_io_data_flag().unwrap().port_encoding_fields(),
                fields,
                "{note}"
            );
        }
    }
}
