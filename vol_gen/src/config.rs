use std::{ffi::OsString, str::FromStr};

use clap::ArgMatches;
use nalgebra::{vector, Vector3};

use crate::{header::HeaderFormat, orders::SampleOrder};

/// Transform `Values` into `Vector`
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

/// App configuration
/// Config is built from args parsed by `clap`
#[derive(Debug)]
pub struct Config {
    /// Dimensions of volume
    pub dims: Vector3<u32>,
    /// Shape of cells
    pub cell_shape: Vector3<f32>,
    /// Type of generator to be used
    pub generator: GeneratorConfig,
    /// Format of header
    pub header_format: HeaderFormat,
    /// Order of samples in file
    pub save_buffer_order: SampleOrder,
    // Output file name
    pub file_name: OsString,
    /// _unimplemented_ Use sparse files to save space
    pub sparse_file: bool,
    /// Optional seed for RNG, to replicate results
    pub seed: Option<u64>,
}

impl Config {
    pub fn from_args(args: ArgMatches) -> Result<Config, String> {
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
                // block size
                let z_val = args.value_of("block-size").unwrap();
                let side = z_val.parse().unwrap();
                // Validate
                for &dim in dims.iter() {
                    let rem = dim % (side as u32);
                    if rem != 0 {
                        return Err(format!(
                            "block-size not divisible by {side} ({dim} % {side} = {rem})"
                        ));
                    }
                }
                SampleOrder::Z(side)
            }
            _ => panic!("Error parsing buffer orser"),
        };
        // File name
        let file_name = args.value_of_os("output-file").unwrap().into(); // Unwrap safe, has default value
                                                                         // Sparse
        let sparse_file = args.is_present("sparse");

        let seed = args.value_of("seed").map(|s| s.parse().unwrap());

        Ok(Config {
            dims,
            cell_shape,
            generator,
            header_format,
            save_buffer_order,
            file_name,
            sparse_file,
            seed,
        })
    }
}

/// Settings specific to generator variant
#[derive(Debug, Clone, Copy)]
pub enum GeneratorConfig {
    /// Generate shapes
    Shapes {
        n_of_shapes: usize,
        sample: u8,
        obj_size: u32,
    },
    /// _unimplemented_ Generate random data
    Noise,
    /// Generate solid volume
    Solid { sample: u8 },
}

impl GeneratorConfig {
    pub fn from_args(args: &ArgMatches) -> GeneratorConfig {
        // Safe to unwrap, args checked by parser
        let s = args.value_of("generator").unwrap();

        // sample
        let sample_str = args.value_of("sample");
        let n_of_shapes_str = args.value_of("n-of-shapes");
        let obj_size_str = args.value_of("object-size");

        match s {
            "shapes" => {
                // Shapes
                let n_of_shapes = n_of_shapes_str.unwrap().parse().unwrap();
                let sample = sample_str.unwrap().parse().unwrap();
                let obj_size = obj_size_str.unwrap().parse().unwrap();
                GeneratorConfig::Shapes {
                    n_of_shapes,
                    sample,
                    obj_size,
                }
            }
            "noise" => {
                // Noise
                GeneratorConfig::Noise
            }
            "solid" => {
                // Solid
                let sample = sample_str.unwrap().parse().unwrap();
                GeneratorConfig::Solid { sample }
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
