mod common;

use common::{read, sensor, timed_sensor, write, ADDRESS, COMMAND_SPACING_US};
use embedded_hal::i2c::{ErrorKind, NoAcknowledgeSource};
use embedded_hal_mock::eh1::i2c::Transaction;
use sht_sensor::{
    AlertLimit, AlertThreshold, ClockStretching, Error, Humidity, Repeatability, Temperature,
};

#[test]
fn measures_and_waits_for_the_measurement() {
    let (mut sensor, mut i2c, mut delay) =
        timed_sensor(&[write(0x2400), read(&[0x6666, 0x8000])], &[15_500]);

    let reading = sensor.measure(Repeatability::High).unwrap();

    assert!((reading.temperature.celsius() - 25.0).abs() < 0.01);
    assert!((reading.humidity.percent() - 50.0).abs() < 0.01);
    i2c.done();
    delay.done();
}

#[test]
fn each_repeatability_waits_its_own_maximum() {
    for (repeatability, command, wait_us) in [
        (Repeatability::Medium, 0x240B, 6_500),
        (Repeatability::Low, 0x2416, 4_500),
    ] {
        let (mut sensor, mut i2c, mut delay) =
            timed_sensor(&[write(command), read(&[0, 0])], &[wait_us]);
        sensor.measure(repeatability).unwrap();
        i2c.done();
        delay.done();
    }
}

#[test]
fn with_clock_stretching_reads_after_only_the_command_spacing() {
    let (mut sensor, mut i2c, mut delay) = timed_sensor(
        &[write(0x2C06), read(&[0x6666, 0x8000])],
        &[COMMAND_SPACING_US],
    );
    sensor
        .measure_with(Repeatability::High, ClockStretching::Enabled)
        .unwrap();
    i2c.done();
    delay.done();
}

#[test]
fn a_corrupted_reading_is_a_crc_error() {
    let (mut sensor, mut i2c) = sensor(&[
        write(0x2400),
        Transaction::read(ADDRESS, vec![0x66, 0x66, 0x00, 0x80, 0x00, 0xA2]),
    ]);
    assert_eq!(sensor.measure(Repeatability::High), Err(Error::Crc));
    i2c.done();
}

#[test]
fn a_missing_sensor_is_an_i2c_error() {
    let nack = ErrorKind::NoAcknowledge(NoAcknowledgeSource::Address);
    let (mut sensor, mut i2c) = sensor(&[write(0x2400).with_error(nack)]);
    assert_eq!(sensor.measure(Repeatability::High), Err(Error::I2c(nack)));
    i2c.done();
}

#[test]
fn switches_the_heater() {
    let (mut sensor, mut i2c) = sensor(&[write(0x306D), write(0x3066)]);
    sensor.enable_heater().unwrap();
    sensor.disable_heater().unwrap();
    i2c.done();
}

#[test]
fn reads_and_clears_the_status_register() {
    let (mut sensor, mut i2c) = sensor(&[write(0xF32D), read(&[0x2010]), write(0x3041)]);

    let status = sensor.status().unwrap();
    sensor.clear_status().unwrap();

    assert!(status.heater_on());
    assert!(status.reset_detected());
    assert!(!status.alert_pending());
    i2c.done();
}

#[test]
fn reads_the_serial_number() {
    let (mut sensor, mut i2c) = sensor(&[write(0x3780), read(&[0x1234, 0xABCD])]);
    assert_eq!(sensor.serial_number().unwrap().value(), 0x1234_ABCD);
    i2c.done();
}

#[test]
fn reads_an_alert_limit() {
    let (mut sensor, mut i2c) = sensor(&[write(0xE11F), read(&[0xCD33])]);
    let limit = sensor.alert_limit(AlertThreshold::HighSet).unwrap();
    assert!((limit.humidity.percent() - 80.0).abs() < 0.5);
    assert!((limit.temperature.celsius() - 60.0).abs() < 0.5);
    i2c.done();
}

#[test]
fn writes_an_alert_limit_with_its_checksum() {
    let limit = AlertLimit::new(Humidity::from_raw(0xCC00), Temperature::from_raw(0x9980));
    let mut bytes = vec![0x61, 0x00];
    bytes.extend(common::frame(&[0xCD33]));
    let (mut sensor, mut i2c) = sensor(&[Transaction::write(ADDRESS, bytes)]);

    sensor
        .set_alert_limit(AlertThreshold::LowSet, limit)
        .unwrap();

    i2c.done();
}

#[test]
fn every_alert_threshold_has_its_own_commands() {
    let thresholds = [
        (AlertThreshold::HighSet, 0xE11F),
        (AlertThreshold::HighClear, 0xE114),
        (AlertThreshold::LowClear, 0xE109),
        (AlertThreshold::LowSet, 0xE102),
    ];
    for (threshold, command) in thresholds {
        let (mut sensor, mut i2c) = sensor(&[write(command), read(&[0])]);
        sensor.alert_limit(threshold).unwrap();
        i2c.done();
    }
}

#[test]
fn soft_reset_waits_for_the_sensor_to_restart() {
    let (mut sensor, mut i2c, mut delay) = timed_sensor(&[write(0x30A2)], &[1_500]);
    sensor.soft_reset().unwrap();
    i2c.done();
    delay.done();
}

#[test]
fn general_call_reset_goes_to_the_general_call_address() {
    let (mut sensor, mut i2c, mut delay) =
        timed_sensor(&[Transaction::write(0x00, vec![0x06])], &[1_500]);
    sensor.general_call_reset().unwrap();
    i2c.done();
    delay.done();
}

#[test]
fn release_hands_back_the_bus() {
    use embedded_hal::i2c::I2c;

    let (sensor, mut i2c) = sensor(&[Transaction::write(0x50, vec![0xAA])]);
    let (mut released, _delay) = sensor.release();
    released.write(0x50, &[0xAA]).unwrap();
    i2c.done();
}
