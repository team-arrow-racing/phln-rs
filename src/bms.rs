//! Battery management system driver.
//!
//! [Product page](https://www.prohelion.com/product-category/bms/)
//! [User's manual](https://www.prohelion.com/wp-content/uploads/2022/07/PHLN67.011v2-BMS-Users-Manual.pdf)

use bitflags::bitflags;
use bxcan::{Frame, Id};

// id offsets for broadcast messages
const ID_BROAD_HEARTBEAT: u16 = 0x00;
const ID_BROAD_CMU_STATUS: u16 = 0x01;
const ID_BROAD_SOC: u16 = 0xF4;
const ID_BROAD_BALANCE_SOC: u16 = 0xF5;
const ID_BROAD_CHG_CTL: u16 = 0xF6;
const ID_BROAD_PRECHARGE: u16 = 0xF7;
const ID_BROAD_MIN_MAX_CELL_VOLT: u16 = 0xF8;
const ID_BROAD_MIN_MAX_CELL_TEMP: u16 = 0xF9;
const ID_BROAD_VOLT_CURR: u16 = 0xFA;
const ID_BROAD_STATUS: u16 = 0xFB;
const ID_BROAD_FAN_STATUS: u16 = 0xFC;
const ID_BROAD_STATUS_EXT: u16 = 0xFD;

#[derive(Debug, Clone, Copy)]
struct CmuStatus {
    serial_number: u32,
    pcb_temperature: u16,
    cell_temperature: u16,
    cell_voltage: [i16; 8],
}

bitflags! {
    /// Error flags
    struct ErrorFlags: u16 {
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
    /// Precharge contactor driver status
    struct ContactorDriverStatus: u8 {
        const CONTACTOR_1_DRIVER_ERROR = 0x01;
        const CONTACTOR_2_DRIVER_ERROR = 0x02;
        const CONTACTOR_1_OUTPUT_ON = 0x04;
        const CONTACTOR_2_OUTPUT_ON = 0x08;
        const SUPPLY_VOLTAGE_OK = 0x10;
        const CONTACTOR_3_DRIVER_ERROR = 0x20;
        const CONTACTOR_3_OUTPUT_ON = 0x40;
        // const UNUSED = 0x80;
    }

}

/// Precharge state
#[derive(Clone, Copy)]
enum PrechargeState {
    Error = 0,
    Idle = 1,
    EnablePack = 5,
    Measure = 2,
    Precharge = 3,
    Run = 4,
}

impl PrechargeState {
    fn from_u8(value: u8) -> Option<PrechargeState> {
        match value {
            0 => Some(PrechargeState::Error),
            1 => Some(PrechargeState::Idle),
            5 => Some(PrechargeState::EnablePack),
            2 => Some(PrechargeState::Measure),
            3 => Some(PrechargeState::Precharge),
            4 => Some(PrechargeState::Run),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Cell {
    cmu: u8,
    number: u8,
}

#[derive(Debug, Clone, Copy)]
struct CellWithVoltage {
    cell: Cell,
    voltage: u16,
}

#[derive(Debug, Clone, Copy)]
struct CellWithTemperature {
    cell: Cell,
    temperature: u16,
}

#[derive(Default, Clone, Copy)]
struct Status {
    device_identifier: Option<u32>,
    device_serial_number: Option<u32>,
    cmu_status: [Option<CmuStatus>; 8],
    soc_amp_hours: Option<f32>,
    soc_percent: Option<f32>,
    balance_soc_amp_hours: Option<f32>,
    balance_soc_percent: Option<f32>,
    charging_cell_voltage_error: Option<u16>,
    cell_temperature_margin: Option<u16>,
    discharging_cell_voltage_error: Option<u16>,
    total_pack_capacity: Option<u16>,
    contactor_driver_status: Option<ContactorDriverStatus>,
    precharge_state: Option<PrechargeState>,
    contactor_supply_voltage: Option<u16>,
    precharge_timer_elapsed: Option<bool>,
    precharge_timer_counter: Option<u8>,
    minimum_voltage_cell: Option<CellWithVoltage>,
    maximum_voltage_cell: Option<CellWithVoltage>,
    minimum_temperature_cell: Option<CellWithTemperature>,
    maximum_temperature_cell: Option<CellWithTemperature>,
    pack_voltage_mv: Option<u32>,
    pack_current_ma: Option<u32>,
}

struct Bmu {
    base_id: u16,

    status: Status,
}

impl Bmu {
    pub fn new(base_id: u16) -> Self {
        Self {
            base_id,
            status: Status {
                ..Default::default()
            },
        }
    }

    pub fn status(self) -> Status {
        self.status
    }

    pub fn receive(&mut self, frame: Frame) -> Result<(), &'static str> {
        match frame.id() {
            Id::Standard(id) => {
                if id.as_raw() >= self.base_id {
                    if let Some(data) = frame.data() {
                        match id.as_raw() - self.base_id {
                            ID_BROAD_HEARTBEAT => {
                                self.status.device_identifier =
                                    Some(u32::from_le_bytes(data[0..4].try_into().unwrap()));
                                self.status.device_serial_number =
                                    Some(u32::from_le_bytes(data[4..8].try_into().unwrap()));
                            }

                            ID_BROAD_SOC => {
                                self.status.soc_amp_hours =
                                    Some(f32::from_le_bytes(data[0..4].try_into().unwrap()));
                                self.status.soc_percent =
                                    Some(f32::from_le_bytes(data[4..8].try_into().unwrap()));
                            }

                            ID_BROAD_BALANCE_SOC => {
                                self.status.balance_soc_amp_hours =
                                    Some(f32::from_le_bytes(data[0..4].try_into().unwrap()));
                                self.status.balance_soc_percent =
                                    Some(f32::from_le_bytes(data[4..8].try_into().unwrap()));
                            }

                            ID_BROAD_CHG_CTL => {
                                self.status.charging_cell_voltage_error =
                                    Some(u16::from_le_bytes(data[0..2].try_into().unwrap()));
                                self.status.cell_temperature_margin =
                                    Some(u16::from_le_bytes(data[2..4].try_into().unwrap()));
                                self.status.discharging_cell_voltage_error =
                                    Some(u16::from_le_bytes(data[4..6].try_into().unwrap()));
                                self.status.discharging_cell_voltage_error =
                                    Some(u16::from_le_bytes(data[6..8].try_into().unwrap()));
                            }

                            ID_BROAD_PRECHARGE => {
                                self.status.contactor_driver_status =
                                    ContactorDriverStatus::from_bits(data[0]);
                                self.status.precharge_state = PrechargeState::from_u8(data[1]);
                                self.status.contactor_supply_voltage =
                                    Some(u16::from_le_bytes(data[2..4].try_into().unwrap()));
                                self.status.precharge_timer_elapsed = Some(data[6] == 1);
                                self.status.precharge_timer_counter = Some(data[7]);
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
}
