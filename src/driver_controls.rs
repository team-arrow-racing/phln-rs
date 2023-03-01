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
