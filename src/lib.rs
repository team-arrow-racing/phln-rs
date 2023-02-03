#![no_std]

use bitflags::bitflags;
use socketcan::CANFrame;

/// Control command.
enum ControlCommand {
    Drive = 0x01,
    Power = 0x02,
    Reset = 0x03,
}

/// Status message.
enum StatusMessage {
    Identification = 0x00,
    Status = 0x01,
    Bus = 0x02,
    Velocity = 0x03,
    PhaseCurrent = 0x04,
    MotorVoltage = 0x05,
    MotorCurrent = 0x06,
    MotorBackEMF = 0x07,
    Rail15V = 0x08,
    Rail3v3And1v9 = 0x09,
    HeatsinkAndMotorTemperature = 0x0B,
    DSPBoardTemperature = 0x0C,
    OdometerAndBusAmpHours = 0x0E,
    SlipSpeedMeasurement = 0x17,
}

/// Configuration commands.
enum ConfigurationCommands {
    ActiveMotorChange = 0x12,
}

/// Motor controller identification information.
struct IdentificationInfo {
    serial_number: u32,
    prohelion_id: u32,
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
    /// Limit flags
    struct LimitFlags: u16 {
        const OUTPUT_VOLTAGE_PWM        = 1 << 0;
        const MOTOR_CURRENT             = 1 << 1;
        const VELOCITY                  = 1 << 2;
        const BUS_CURRENT               = 1 << 3;
        const BUS_VOLTAGE_UPPER_LIMIT   = 1 << 4;
        const BUS_VOLTAGE_LOWER_LIMIT   = 1 << 5;
        const TEMPERATURE               = 1 << 6;
    }
}

enum VoltageRail {
    /// 1.9V rail.
    _1V9,
    /// 3.3V rail.
    _3V3,
    /// 15V rail.
    _15V
}

enum TemperatureSensor {
    /// Heatsink sensor.
    Heatsink,
    /// Motor sensor.
    Motor,
    /// DSP board sensor.
    DspBoard,
}

/// Active motor change magic string.
static MOTOR_CHANGE: &[u8] = "ACTMOT".as_bytes();

type Confirmation = Result<(), &'static str>;

trait WaveSculptor {
    // Drive commands.

    /// Set the desired current setpoint as a percentage of the maximum current
    /// setting and desired velocity in RPM.
    fn drive(current: f32, velocity: f32) -> Confirmation;

    /// Set desired current draw from the bus by the controller as a percentage
    /// of absolute bus current limit.
    fn power(bus_current: f32) -> Confirmation;

    /// Reset the software on the WaveSculptor.
    fn reset() -> Confirmation;

    // Status commands.

    /// Identification information.
    fn serial_number() -> Result<u32, &'static str>;

    /// Prohelion (or Tritium) identifier.
    fn manufacturer_id() -> Result<u32, &'static str>;

    /// CAN receive error count.
    fn receive_error_count() -> Result<u8, &'static str>;

    /// CAN transmission error count.
    fn transmit_error_count() -> Result<u8, &'static str>;

    /// Currently selected motor profile.
    fn active_motor() -> Result<u16, &'static str>;

    /// Error flag status.
    fn error_flags() -> Result<ErrorFlags, &'static str>;

    /// Limit flag status.
    fn limit_flags() -> Result<LimitFlags, &'static str>;

    /// Bus current in amps.
    fn bus_current() -> Result<f32, &'static str>;

    /// Bus voltage in volts.
    fn bus_voltage() -> Result<f32, &'static str>;

    /// Vehicle velocity in meters per seccond.
    fn vehicle_velocity() -> Result<f32, &'static str>;

    /// Motor velocity in RPM.
    fn motor_velocity() -> Result<f32, &'static str>;

    /// RMS current in motor phase C in amps.
    fn phase_current_c() -> Result<f32, &'static str>;

    /// RMS current in motor phase B in amps.
    fn phase_current_b() -> Result<f32, &'static str>;

    /// Voltage vector of the motor in volts.
    fn motor_voltage_vector() -> Result<num_complex::Complex32, &'static str>;

    /// Current vector fo the motor in amps.
    fn motor_current_vector() -> Result<num_complex::Complex32, &'static str>;

    /// Motor back EMF measurement in volts.
    fn motor_back_emf() -> Result<num_complex::Complex32, &'static str>;

    /// Voltage rail measurement in volts.
    fn rail_measurement(rail: VoltageRail) -> Result<f32, &'static str>;

    /// Temperature sensor measurement in degrees celcius.
    fn temperature_measurement(sensor: TemperatureSensor) -> Result<f32, &'static str>;

    /// Bus amp-hour consumption.
    fn bus_amp_hours() -> Result<f32, &'static str>;

    /// Odometer measurement in meters since last reset.
    fn odometer_measurement() -> Result<f32, &'static str>;

    /// Slip speed measurement in Hz.
    fn slip_speed_measurement() -> Result<f32, &'static str>;

    // Configuration commands.

    /// Change active motor
    fn active_motor_change(motor: u8) -> Result<f32, &'static str>;

    // Interface helpers.

    /// Send CAN bus frame.
    fn send_frame(frame: CANFrame) -> Confirmation;
}
