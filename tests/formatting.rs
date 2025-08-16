use itertools::izip;
use pacman::systems::formatting::format_timing_display;
use smallvec::SmallVec;
use std::time::Duration;

use pretty_assertions::assert_eq;

fn get_timing_data() -> Vec<(String, Duration, Duration)> {
    vec![
        ("total".to_string(), Duration::from_micros(1234), Duration::from_micros(570)),
        ("input".to_string(), Duration::from_micros(120), Duration::from_micros(45)),
        ("player".to_string(), Duration::from_micros(456), Duration::from_micros(123)),
        ("movement".to_string(), Duration::from_micros(789), Duration::from_micros(234)),
        ("render".to_string(), Duration::from_micros(12), Duration::from_micros(3)),
        ("debug".to_string(), Duration::from_nanos(460), Duration::from_nanos(557)),
    ]
}

fn get_formatted_output() -> impl IntoIterator<Item = String> {
    format_timing_display(get_timing_data())
}

#[test]
fn test_formatting_alignment() {
    let mut colon_positions = vec![];
    let mut first_decimal_positions = vec![];
    let mut second_decimal_positions = vec![];
    let mut first_unit_positions = vec![];
    let mut second_unit_positions = vec![];

    get_formatted_output().into_iter().for_each(|line| {
        let (mut got_decimal, mut got_unit) = (false, false);
        for (i, char) in line.chars().enumerate() {
            match char {
                ':' => colon_positions.push(i),
                '.' => {
                    if got_decimal {
                        second_decimal_positions.push(i);
                    } else {
                        first_decimal_positions.push(i);
                    }
                    got_decimal = true;
                }
                's' => {
                    if got_unit {
                        first_unit_positions.push(i);
                    } else {
                        second_unit_positions.push(i);
                        got_unit = true;
                    }
                }
                _ => {}
            }
        }
    });

    // Assert that all positions were found
    assert_eq!(
        vec![
            &colon_positions,
            &first_decimal_positions,
            &second_decimal_positions,
            &first_unit_positions,
            &second_unit_positions
        ]
        .iter()
        .all(|p| p.len() == 6),
        true
    );

    // Assert that all positions are the same
    assert!(
        colon_positions.iter().all(|&p| p == colon_positions[0]),
        "colon positions are not the same {:?}",
        colon_positions
    );
    assert!(
        first_decimal_positions.iter().all(|&p| p == first_decimal_positions[0]),
        "first decimal positions are not the same {:?}",
        first_decimal_positions
    );
    assert!(
        second_decimal_positions.iter().all(|&p| p == second_decimal_positions[0]),
        "second decimal positions are not the same {:?}",
        second_decimal_positions
    );
    assert!(
        first_unit_positions.iter().all(|&p| p == first_unit_positions[0]),
        "first unit positions are not the same {:?}",
        first_unit_positions
    );
    assert!(
        second_unit_positions.iter().all(|&p| p == second_unit_positions[0]),
        "second unit positions are not the same {:?}",
        second_unit_positions
    );
}
