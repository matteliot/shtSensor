mod common;

use common::{read, sensor, write, ADDRESS};
use embedded_hal::i2c::{ErrorKind, NoAcknowledgeSource};
use embedded_hal_mock::eh1::i2c::Transaction;
use sht_sensor::{Error, Rate, Repeatability};

#[test]
fn starts_fetches_and_stops() {
    let (sensor, mut i2c) = sensor(&[
        write(0x2130),
        write(0xE000),
        read(&[0x6666, 0x8000]),
        write(0x3093),
        write(0x2400),
        read(&[0, 0]),
    ]);

    let mut periodic = sensor
        .start_periodic(Rate::Hz1, Repeatability::High)
        .unwrap();
    let reading = periodic.fetch().unwrap().expect("a reading was ready");
    let mut single_shot = periodic.stop().unwrap();
    single_shot.measure(Repeatability::High).unwrap();

    assert!((reading.temperature.celsius() - 25.0).abs() < 0.01);
    i2c.done();
}

#[test]
fn fetch_reports_no_new_data_as_none() {
    let nack = ErrorKind::NoAcknowledge(NoAcknowledgeSource::Data);
    let (sensor, mut i2c) = sensor(&[
        write(0x2B32),
        write(0xE000),
        Transaction::read(ADDRESS, vec![0; 6]).with_error(nack),
    ]);

    let mut periodic = sensor.start_art().unwrap();

    assert_eq!(periodic.fetch(), Ok(None));
    i2c.done();
}

#[test]
fn fetch_passes_other_bus_errors_through() {
    let (sensor, mut i2c) = sensor(&[
        write(0x2032),
        write(0xE000),
        Transaction::read(ADDRESS, vec![0; 6]).with_error(ErrorKind::Bus),
    ]);

    let mut periodic = sensor
        .start_periodic(Rate::HalfHz, Repeatability::High)
        .unwrap();

    assert_eq!(periodic.fetch(), Err(Error::I2c(ErrorKind::Bus)));
    i2c.done();
}

#[test]
fn a_failed_start_hands_the_driver_back_in_single_shot_mode() {
    let (sensor, mut i2c) = sensor(&[
        write(0x2737).with_error(ErrorKind::Bus),
        write(0x2400),
        read(&[0, 0]),
    ]);

    let Err(failure) = sensor.start_periodic(Rate::Hz10, Repeatability::High) else {
        panic!("the start command should have failed")
    };
    let mut sensor = failure.driver;
    sensor.measure(Repeatability::High).unwrap();

    assert_eq!(failure.error, Error::I2c(ErrorKind::Bus));
    i2c.done();
}

#[test]
fn soft_reset_ends_periodic_mode() {
    let (sensor, mut i2c) = sensor(&[write(0x2334), write(0x30A2), write(0xF32D), read(&[0x0010])]);

    let periodic = sensor
        .start_periodic(Rate::Hz4, Repeatability::High)
        .unwrap();
    let mut sensor = periodic.soft_reset().unwrap();

    assert!(sensor.status().unwrap().reset_detected());
    i2c.done();
}
