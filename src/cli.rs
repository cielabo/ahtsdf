use std::process;
use clap::{Arg, App};

pub struct InputData {
    pub bruker_path: String,
    pub shader: String,
    pub delta_t: f64,
    pub amplitude: f64,
    pub resolution: [u32; 2],
}
impl InputData {
    pub fn from_cli() -> InputData{
        let matches = App::new("ahtsdf")
                                .about("Real-time rendering of AHT curves from .bruker files")
                                .version("0.1.0")
                                .arg(Arg::with_name("bruker_path")
                                    .required(true)
                                    .short("p")
                                    .long("path")
                                    .value_name("BRUKER_PATH")
                                    .takes_value(true)
                                    .help("the path of your .bruker file [this is required]"))
                                .arg(Arg::with_name("delta_t")
                                    .short("d")
                                    .long("delta_t")
                                    .value_name("DELTA_T")
                                    .takes_value(true)
                                    .allow_hyphen_values(true)
                                    .help("subpulse time interval, in microseconds [default: 25]"))
                                .arg(Arg::with_name("amplitude")
                                    .short("a")
                                    .long("amplitude")
                                    .value_name("RF_AMPLITUDE")
                                    .takes_value(true)
                                    .allow_hyphen_values(true)
                                    .help("pulse RF amplitude, in Hz [default: 10000]"))
                                .arg(Arg::with_name("resolution")
                                    .short("r")
                                    .long("resolution")
                                    .value_name("WIDTH HEIGHT")
                                    .takes_value(true)
                                    .number_of_values(2)
                                    .help("window dimensions, separated by a space [default: 500 500]"))
                                .arg(Arg::with_name("shader")
                                    .short("s")
                                    .long("shader")
                                    .value_name("SHADER_MODE")
                                    .takes_value(true)
                                    .help("shader mode, options are: DEFAULT, CHEAP, RAD_VEC, DISC")
                            )
                            .get_matches();

        let bruker_path: &str;
        let shader: &str;
        let delta_t: f64;
        let amplitude: f64;
        let mut resolution: [u32; 2];
        

        bruker_path = matches.value_of("bruker_path").unwrap();
        delta_t = match matches.is_present("delta_t") {
            true => match matches.value_of("delta_t").unwrap().parse::<f64>() {
                Ok(x) => x * 0.000001,
                Err(_) => {eprintln!("Error: Could not parse delta_t as floating point number"); process::exit(1)}
            }
            false => 25.0 * 0.000001,
        };
        amplitude = match matches.is_present("amplitude") {
            true => match matches.value_of("amplitude").unwrap().parse::<f64>() {
                Ok(x) => x,
                Err(_) => {eprintln!("Error: Could not parse amplitude as floating point number"); process::exit(1)}
            }
            false => 10000.0f64
        };
        resolution = match matches.is_present("resolution") {
            true => {
                let s: Vec<&str> = matches.values_of("resolution").unwrap().collect();
                resolution = [0, 0];
                let mut count  = 0;
                for value in s {
                    match value.parse::<u32>() {
                        Ok(x) => {resolution[count] = x; count += 1;},
                        Err(_) => {eprintln!("Error: Could not parse resolution arguments"); process::exit(1)}
                    }
                }
                resolution
            }
            false => [500, 500]
        };
        shader = match matches.is_present("shader") {
            true => {
                match matches.value_of("shader").unwrap() {
                    "DEFAULT" => "shaders/default.glsl",
                    "CHEAP" => "shaders/cheap.glsl",
                    "RAD_VEC" => "shaders/rad_vec.glsl",
                    "AMBIENCE" => "shaders/ambience.glsl",
                    "DISC" => "shaders/disc.glsl",
                    _ => {println!("Could not parse shader argument, defaulting to DEFAULT"); "shaders/default.glsl"}
                }
            }
            false => "shaders/default.glsl"
        };
        
        InputData{bruker_path: String::from(bruker_path), shader: String::from(shader), delta_t, amplitude, resolution}

    }
}