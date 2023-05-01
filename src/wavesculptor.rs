//! WaveScultpor 22 and 200 motor driver.
//!
//! This driver is backwards compaible with Tritium WaveSculptors.

use bitflags::bitflags;
use bxcan::{Frame, Id, StandardId};
use num_complex::Complex32;

// broadcase message identifiers normalized for base id.
const ID_BROAD_ID: u16 = 0x00;
const ID_BROAD_STATUS: u16 = 0x01;
const ID_BROAD_BUS_MEAS: u16 = 0x02;
const ID_BROAD_VELOCITY: u16 = 0x03;
const ID_BROAD_PHASE_CURRENT: u16 = 0x04;
const ID_BROAD_MOTOR_VOLTAGE: u16 = 0x05;
const ID_BROAD_MOTOR_CURRENT: u16 = 0x06;
const ID_BROAD_BACK_EMF: u16 = 0x07;
const ID_BROAD_RAIL_15V: u16 = 0x08;
const ID_BROAD_RAIL_3V3_1V9: u16 = 0x09;
const ID_BROAD_TEMP_HSINK_MOTOR: u16 = 0x0B;
const ID_BROAD_TEMP_DSP: u16 = 0x0C;
const ID_BROAD_ODOMETER: u16 = 0x0E;
const ID_BROAD_SLIP_SPEED: u16 = 0x17;

// command message identifiers normalized for base id.
const ID_CMD_MOTOR_CHANGE: u16 = 0x12;

/// Default base identifier
pub static ID_BASE: u16 = 0x600;

bitflags! {
    /// Error flags
    pub struct ErrorFlags: u16 {
        const HARDWARE_OVER_CURRENT       = 1 << 0;
        const SOFTWARE_OVER_CURRENT       = 1 << 1;
        const DC_BUS_OVER_CURRENT         = 1 << 2;
        const BAD_MOTOR_POSITION_SEQUENCE = 1 << 3;
        const WATCHDOG_CAUSED_LAST_RESET  = 1 << 4;
        const CONFIG_READ_ERROR           = 1 << 5;
        const RAIL_15V_UVLO               = 1 << 6;
        const DESATURATION_FAULT          = 1 << 7;
        const MOTOT_OVER_SPEED            = 1 << 8;
    }
}

bitflags! {
    /// Limit flags
    pub struct LimitFlags: u16 {
        const OUTPUT_VOLTAGE_PWM        = 1 << 0;
        const MOTOR_CURRENT             = 1 << 1;
        const VELOCITY                  = 1 << 2;
        const BUS_CURRENT               = 1 << 3;
        const BUS_VOLTAGE_UPPER_LIMIT   = 1 << 4;
        const BUS_VOLTAGE_LOWER_LIMIT   = 1 << 5;
        const TEMPERATURE               = 1 << 6;
    }
}

/// Status
#[derive(Default, Clone, Copy)]
pub struct Status {
    /// Device serial number, allocated at manufacture
    serial_number: Option<u32>,
    /// Device identifier (Tritium ID or Prohelion ID)
    identifier: Option<u32>,
    /// CAN receive error count
    can_rx_error_count: Option<u8>,
    /// CAN transmit error count
    can_tx_error_count: Option<u8>,
    /// Active motor identifier
    active_motor: Option<u16>,
    /// Error flags
    error_flags: Option<ErrorFlags>,
    /// Limit flags
    limit_flags: Option<LimitFlags>,
    /// Bus current in amps
    bus_current: Option<f32>,
    /// Bus voltage in volts
    bus_voltage: Option<f32>,
    /// Vehicle velocity in meters/second
    vehicle_velocity: Option<f32>,
    /// Motor velocity in RPM
    motor_velocity: Option<f32>,
    /// Phase C current in amps RMS
    phase_c_current: Option<f32>,
    /// Phase B current in amps RMS
    phase_b_current: Option<f32>,
    /// Motor voltage vector in volts
    motor_voltage_vector: Option<Complex32>,
    /// Motor current vector in volts
    motor_current_vector: Option<Complex32>,
    /// Motor back-EMF vector in volts
    motor_back_emf_vector: Option<Complex32>,
    /// 15V rail measurement in volts
    rail_15v: Option<f32>,
    /// 3.3V rail measurement in volts
    rail_3v3: Option<f32>,
    /// 1.9V rail measurement in volts
    rail_1v9: Option<f32>,
    /// Heat-sink temperature in degrees celcius
    heatsink_temperature: Option<f32>,
    /// Motor temperature in degrees celcius
    motor_temperature: Option<f32>,
    /// DSP board temperature in degrees celcius
    dsp_board_temperature: Option<f32>,
    /// DC bus amp-hours measurement
    bus_amp_hours: Option<f32>,
    /// Odometer (distance traveled since last reset) in meters.
    odometer: Option<f32>,
    /// Slip speed measurement in Hz
    slip_speed: Option<f32>,
}

pub struct WaveSculptor {
    base_id: u16,

    status: Status,
}

impl WaveSculptor {
    /// Create a new WaveSculptor instance.
    pub fn new(base_id: u16) -> Self {
        Self {
            base_id,
            status: Status {
                ..Default::default()
            },
        }
    }

    /// Get the current status state of the device
    pub fn status(self) -> Status {
        self.status
    }

    pub fn receive(&mut self, frame: Frame) -> Result<(), &'static str> {
        match frame.id() {
            Id::Standard(id) => {
                // is within range
                if id.as_raw() >= self.base_id {
                    // is there some data in this frame?
                    if let Some(data) = frame.data() {
                        // normalized identifier
                        match id.as_raw() - self.base_id {
                            ID_BROAD_ID => {
                                self.status.identifier =
                                    Some(u32::from_le_bytes(data[0..4].try_into().unwrap()));
                                self.status.serial_number =
                                    Some(u32::from_le_bytes(data[4..8].try_into().unwrap()));
                            }

                            ID_BROAD_STATUS => {
                                self.status.can_rx_error_count = Some(data[0]);
                                self.status.can_tx_error_count = Some(data[1]);
                                self.status.active_motor =
                                    Some(u16::from_le_bytes(data[2..4].try_into().unwrap()));
                                self.status.error_flags = ErrorFlags::from_bits(
                                    u16::from_le_bytes(data[4..6].try_into().unwrap()),
                                );
                                self.status.limit_flags = LimitFlags::from_bits(
                                    u16::from_le_bytes(data[6..8].try_into().unwrap()),
                                );
                            }

                            ID_BROAD_BUS_MEAS => {
                                self.status.bus_voltage =
                                    Some(f32::from_le_bytes(data[0..4].try_into().unwrap()));
                                self.status.bus_current =
                                    Some(f32::from_le_bytes(data[4..8].try_into().unwrap()));
                            }

                            ID_BROAD_VELOCITY => {
                                self.status.motor_velocity =
                                    Some(f32::from_le_bytes(data[0..4].try_into().unwrap()));
                                self.status.vehicle_velocity =
                                    Some(f32::from_le_bytes(data[4..8].try_into().unwrap()));
                            }

                            ID_BROAD_PHASE_CURRENT => {
                                self.status.phase_b_current =
                                    Some(f32::from_le_bytes(data[0..4].try_into().unwrap()));
                                self.status.phase_c_current =
                                    Some(f32::from_le_bytes(data[4..8].try_into().unwrap()));
                            }

                            ID_BROAD_MOTOR_VOLTAGE => {
                                let i = f32::from_le_bytes(data[0..4].try_into().unwrap());
                                let r = f32::from_le_bytes(data[4..8].try_into().unwrap());
                                self.status.motor_voltage_vector = Some(Complex32::new(r, i));
                            }

                            ID_BROAD_MOTOR_CURRENT => {
                                let i = f32::from_le_bytes(data[0..4].try_into().unwrap());
                                let r = f32::from_le_bytes(data[4..8].try_into().unwrap());
                                self.status.motor_current_vector = Some(Complex32::new(r, i));
                            }

                            ID_BROAD_BACK_EMF => {
                                let i = f32::from_le_bytes(data[0..4].try_into().unwrap());
                                let r = f32::from_le_bytes(data[4..8].try_into().unwrap());
                                self.status.motor_back_emf_vector = Some(Complex32::new(r, i));
                            }

                            ID_BROAD_RAIL_15V => {
                                self.status.rail_15v =
                                    Some(f32::from_le_bytes(data[4..8].try_into().unwrap()));
                            }

                            ID_BROAD_RAIL_3V3_1V9 => {
                                self.status.rail_1v9 =
                                    Some(f32::from_le_bytes(data[0..4].try_into().unwrap()));
                                self.status.rail_3v3 =
                                    Some(f32::from_le_bytes(data[4..8].try_into().unwrap()));
                            }

                            ID_BROAD_TEMP_HSINK_MOTOR => {
                                self.status.motor_temperature =
                                    Some(f32::from_le_bytes(data[0..4].try_into().unwrap()));
                                self.status.heatsink_temperature =
                                    Some(f32::from_le_bytes(data[4..8].try_into().unwrap()));
                            }

                            ID_BROAD_TEMP_DSP => {
                                self.status.dsp_board_temperature =
                                    Some(f32::from_le_bytes(data[0..4].try_into().unwrap()));
                            }

                            ID_BROAD_ODOMETER => {
                                self.status.odometer =
                                    Some(f32::from_le_bytes(data[0..4].try_into().unwrap()));
                                self.status.bus_amp_hours =
                                    Some(f32::from_le_bytes(data[4..8].try_into().unwrap()));
                            }

                            ID_BROAD_SLIP_SPEED => {
                                self.status.slip_speed =
                                    Some(f32::from_le_bytes(data[4..8].try_into().unwrap()));
                            }

                            _ => {}
                        }
                    }
                }
            }
            Id::Extended(_) => {}
        }

        Ok(())
    }

    /// Change the active motor profile.
    ///
    /// `motor` must be between 0 and 9 (inclusive).
    pub fn active_motor_change(self, motor: u8) -> Frame {
        assert!(motor <= 9);

        let id = StandardId::new(self.base_id + ID_CMD_MOTOR_CHANGE).unwrap();

        Frame::new_data(id, [0, motor, b'A', b'C', b'T', b'M', b'O', b'T'])
    }
}
