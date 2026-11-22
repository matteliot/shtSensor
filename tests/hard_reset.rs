mod common;

use common::write;
use embedded_hal_mock::eh1::delay::{CheckedDelay, Transaction as DelayTransaction};
use embedded_hal_mock::eh1::digital::{Mock as PinMock, State, Transaction as PinTransaction};
use embedded_hal_mock::eh1::i2c::Mock as I2cMock;
use sht_sensor::{Address, Rate, Repeatability, Sht3x};

#[test]
fn pulses_the_reset_pin_then_waits_for_the_sensor() {
    let mut i2c = I2cMock::new(&[]);
    let mut delay = CheckedDelay::new(&[
        DelayTransaction::delay_ns(1_000),
        DelayTransaction::delay_us(1_500),
    ]);
    let mut sensor = Sht3x::new(i2c.clone(), delay.clone(), Address::Low);
    let mut pin = PinMock::new(&[
        PinTransaction::set(State::Low),
        PinTransaction::set(State::High),
    ]);

    sensor.hard_reset(&mut pin).unwrap();

    pin.done();
    i2c.done();
    delay.done();
}

#[test]
fn hard_reset_ends_periodic_mode() {
    let i2c = I2cMock::new(&[write(0x2130)]);
    let mut done = i2c.clone();
    let delay = CheckedDelay::new(&[
        DelayTransaction::delay_us(1_000),
        DelayTransaction::delay_ns(1_000),
        DelayTransaction::delay_us(1_500),
    ]);
    let mut delay_done = delay.clone();
    let mut pin = PinMock::new(&[
        PinTransaction::set(State::Low),
        PinTransaction::set(State::High),
    ]);

    let periodic = Sht3x::new(i2c, delay, Address::Low)
        .start_periodic(Rate::Hz1, Repeatability::High)
        .unwrap();
    let _single_shot: Sht3x<_, _> = periodic.hard_reset(&mut pin).unwrap();

    pin.done();
    done.done();
    delay_done.done();
}
