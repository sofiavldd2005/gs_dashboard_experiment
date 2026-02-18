use serde::{Deserialize, Serialize};

///Struct of data to be received from the rocket
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Telemetry {
    pub Yaw: f32,
    pub Pitch: f32,
    pub Roll: f32,
    pub Temperature: u16,
    pub Pressure: u16,
    pub AccelZ: f32,
    pub GyroX: f32,
    pub GyroY: f32,
    pub GyroZ: f32,
    pub QuatX: f32,
    pub QuatY: f32,
    pub QuatZ: f32,
    pub QuatS: f32,
    pub Lat: f32,
    pub Lon: f32,
    pub State: u8,
}

///Enum of the CMDS for future use, possible commands to be sent from the gs
//TODO
#[derive(Deserialize, Debug, Serialize, Clone)]
pub enum Cmds {
    ABORT,
    ARM,
    PING,
    LAUCH,
}
