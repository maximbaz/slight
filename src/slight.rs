use strum::IntoEnumIterator;

use std::path::PathBuf;

use crate::{
    class::Class,
    device::Device,
    error::{Result, SlightError},
    io::IO,
    range::{Range, RangeBuilder},
    value::{Input, Value},
    Args,
};

const EXPONENT_DEFAULT: f32 = 4.0;
// TODO: std::time::Duration::from_secs_f32 is not stable as const fn yet
const SLEEP_DURATION_DEFAULT: f32 = 1.0 / 30.0;

pub struct Slight {
    device: Device,
    exponent: Option<f32>,
    input: Input,
    stdout: bool,
    verbose: bool,
}

impl Slight {
    pub fn set_brightness(&mut self) -> Result<()> {
        let curr = self.device.brightness.as_value();
        let max = self.device.brightness.max();
        let range = Self::create_range(curr, &self.input, max, self.exponent);
        if self.stdout {
            Self::print_range(range);
            return Ok(());
        }
        self.set_brightness_range(range)?;
        Ok(())
    }

    fn scan_devices() -> Result<Vec<Device>> {
        let mut devices = Vec::new();
        Class::iter().map(|c| PathBuf::from(&c)).for_each(|class| {
            IO::scan(&class).map_or_else(
                //TODO: print only if self.verbose
                |e| eprintln!("Failed to read class: {}", e),
                |ids| {
                    for id in ids {
                        class.join(id).as_path().try_into().map_or_else(
                            //TODO: print only if self.verbose
                            |e| eprintln!("Failed to read device: {}", e),
                            |device| devices.push(device),
                        )
                    }
                },
            )
        });
        Ok(devices)
    }

    pub fn print_devices() -> Result<()> {
        let devices = Self::scan_devices()?;
        if devices.is_empty() {
            println!("No devices found!");
        } else {
            println!("Found devices:");
            for dev in devices {
                println!("\t{}", dev);
            }
        }
        Ok(())
    }

    fn print_range(r: Box<dyn RangeBuilder>) {
        println!(
            "{}",
            r.build()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    fn create_range(
        curr: usize,
        input: &Input,
        max: usize,
        exponent: Option<f32>,
    ) -> Box<dyn RangeBuilder> {
        // TODO: Range::try_from(input)?
        let r = match &input {
            Input::To(Value::Relative(p)) => Range::new(curr, max).to().relative(*p),
            Input::To(Value::Absolute(v)) => Range::new(curr, max).to().absolute(*v as isize),
            Input::By(s, Value::Absolute(v)) => Range::new(curr, max)
                .by()
                .absolute((s * *v as f32) as isize),
            Input::By(s, Value::Relative(p)) => Range::new(curr, max).by().relative(s * *p),
        };

        if let Some(e) = exponent {
            Box::new(r.exp(e))
        } else {
            Box::new(r)
        }
    }

    fn set_brightness_range(&mut self, range: Box<dyn RangeBuilder>) -> Result<()> {
        let path = self.device.my_path();
        for v in range.build() {
            self.device.brightness.set(v, &path)?;
            std::thread::sleep(std::time::Duration::from_secs_f32(SLEEP_DURATION_DEFAULT));
        }
        Ok(())
    }

    fn select_device<'a>(devices: &'a [Device], id: Option<&'a str>) -> Result<&'a Device> {
        if let Some(id) = id {
            Self::find_device(devices, id).ok_or(SlightError::NoSpecifiedDeviceFound)
        } else {
            Self::default_device(devices).ok_or(SlightError::NoSuitableDeviceFound)
        }
    }

    fn find_device<'a>(devices: &'a [Device], id: &'a str) -> Option<&'a Device> {
        devices.iter().find(|&d| d.id == id)
    }

    fn default_device(devices: &[Device]) -> Option<&Device> {
        devices.iter().find(|&d| d.class == Class::Backlight)
    }
}

impl TryFrom<&Args> for Slight {
    type Error = SlightError;

    fn try_from(a: &Args) -> std::result::Result<Self, Self::Error> {
        let devices = Self::scan_devices()?;
        // TODO: any reasons to pass a reference?
        let device = Self::select_device(&devices, a.id.as_deref())?;
        let exponent = match a.exponent {
            None => None,
            Some(None) => Some(EXPONENT_DEFAULT),
            Some(Some(v)) => Some(v),
        };
        Ok(Self {
            device: device.clone(),
            exponent,
            input: Input::try_from(a.input.as_ref().ok_or(SlightError::NoInput)?.as_str()).unwrap(),
            stdout: a.stdout,
            verbose: a.verbose,
        })
    }
}
