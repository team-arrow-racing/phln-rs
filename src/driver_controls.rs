//! EV Driver Controls
//!
//! This module lets you emulate driver controls messages to, for example,
//! control a BMU or WaveSculptor.

use bxcan::{Frame, StandardId};

/// Default base identifier value
pub const ID_BASE_DEFAULT: u16 = 0x500;

/// Ignition position options
#[derive(Debug, Clone, Copy)]
pub enum IgnitionPosition {
    Run,
    Start,
}

/// EV Driver Controls
#[derive(Debug, Clone, Copy)]
pub struct DriverControls {
    base_id: u16,
}

impl DriverControls {
    /// Create a new driver controls instance.
    pub fn new(base_id: u16) -> Self {
        Self { base_id }
    }

    /// Motor drive command
    pub fn motor_drive(self, velocity_rpm: f32, current_percent: f32) -> Frame {
        let id = StandardId::new(self.base_id + 0x01).unwrap();

        let vel = velocity_rpm.to_le_bytes();
        let cur = current_percent.to_be_bytes();

        let data = [
            vel[0], vel[1], vel[2], vel[3], cur[0], cur[1], cur[2], cur[3],
        ];

        Frame::new_data(id, data)
    }

    /// Motor power command
    pub fn motor_power(self, bus_current_percent: f32) -> Frame {
        let id = StandardId::new(self.base_id + 0x02).unwrap();

        let bus = bus_current_percent.to_le_bytes();

        let data = [0, 0, 0, 0, bus[0], bus[1], bus[2], bus[3]];

        Frame::new_data(id, data)
    }

    /// Reset WaveSculptor
    pub fn reset_wavesculptor(self) -> Frame {
        let id = StandardId::new(self.base_id + 0x03).unwrap();

        Frame::new_data(id, [0; 8])
    }

    /// Form a switch position frame
    pub fn switch_position(self, ignition_position: IgnitionPosition) -> Frame {
        let id = StandardId::new(self.base_id + 0x05).unwrap();

        let data: u8 = match ignition_position {
            IgnitionPosition::Run => 0x0020,
            IgnitionPosition::Start => 0x0040,
        };

        // only first byte is occupied, manual shows all bytes used
        Frame::new_data(id, [data, 0, 0, 0, 0, 0, 0, 0])
    }
}
