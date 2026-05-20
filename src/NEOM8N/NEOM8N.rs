#![no_std]
#![no_main]

pub struct GpsData {

    // LLA position
    pub lat: f64, // Degrees
    pub long: f64, // Degrees

    // Heading
    pub heading: f64, // Degrees
    pub velocity: f64, // m/s
}

use stm32f1xx_hal::prelude::*;


struct