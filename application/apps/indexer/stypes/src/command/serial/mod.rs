#[cfg(test)]
mod proptest;

use crate::*;

/// Represents a list of serial ports.
///
/// This structure contains a vector of strings, where each string represents the name
/// or identifier of a serial port available on the system.
#[derive(Clone, Serialize, Deserialize, Debug)]
#[extend::encode_decode]
#[cfg_attr(
    all(test, feature = "test_and_gen"),
    derive(TS),
    ts(export, export_to = "command.ts")
)]
pub struct SerialPortsList(pub Vec<String>);
