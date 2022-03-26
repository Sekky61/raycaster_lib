use std::{ffi::OsString, str::FromStr};

use clap::ArgMatches;
use nalgebra::{vector, Vector3};

use crate::sample_order::{HeaderFormat, SampleOrder};

// todo Describe command args here

fn values_to_vector3<T>(args: &ArgMatches, key: &str) -> Vector3<T>
where
    T: FromStr + Copy,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    let vals: Vec<T> = args
        .values_of(key)
        .unwrap()
        .into_iter()
        .map(|v| v.parse::<T>().expect("Parse error"))
        .collect();
    vector![vals[0], vals[1], vals[2]]
}

#[derive(Debug)]
pub struct Config {
    pub dims: Vector3<u32>,
    pub cell_shape: Vector3<f32>,
    pub generator: GeneratorConfig,
    pub header_format: HeaderFormat,
    pub save_buffer_order: SampleOrder,
    pub file_name: OsString,
    pub sparse_file: bool,
}

impl From<ArgMatches> for Config {
    fn from(args: ArgMatches) -> Self {
        // Dims
        let dims = values_to_vector3(&args, "dims");
        // Cell shape
        let cell_shape = values_to_vector3(&args, "shape");
        // Generator
        let generator = GeneratorConfig::from_args(&args);
        // Header
        let header_format = HeaderFormat::Default;
        // Order linear/z
        let layout = args.value_of("layout").unwrap();
        let save_buffer_order = match layout {
            "linear" => SampleOrder::Linear,
            "z" => {
                let z_val = args.value_of("block-size").unwrap();
                let side = z_val.parse().unwrap();
                SampleOrder::Z(side)
            }
            _ => panic!("Error parsing buffer orser"),
        };
        // File name
        let file_name = args.value_of_os("output-file").unwrap().into(); // Unwrap safe, has default value
                                                                         // Sparse
        let sparse_file = args.is_present("sparse");

        Config {
            dims,
            cell_shape,
            generator,
            header_format,
            save_buffer_order,
            file_name,
            sparse_file,
        }
    }
}

// Enum variant has settings specific to generator variant
#[derive(Debug)]
pub enum GeneratorConfig {
    Shapes,
    Noise,
    Solid,
}

impl GeneratorConfig {
    pub fn from_args(args: &ArgMatches) -> GeneratorConfig {
        // Safe to unwrap, args checked by parser
        let s = args.value_of("generator").unwrap();

        match s {
            "shapes" => {
                // Shapes
                GeneratorConfig::Shapes
            }
            "noise" => {
                // Noise
                GeneratorConfig::Noise
            }
            "solid" => {
                // Solid
                GeneratorConfig::Solid
            }
            _ => panic!("Error parsing generator config"),
        }
    }
}

#[derive(Debug)]
pub enum GeneratorType {
    Shapes,
    Noise,
    Solid,
}

impl FromStr for GeneratorType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "shapes" => Ok(GeneratorType::Shapes),
            "noise" => Ok(GeneratorType::Noise),
            "solid" => Ok(GeneratorType::Solid),
            _ => Err(()),
        }
    }
}
