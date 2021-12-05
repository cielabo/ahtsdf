extern crate glow;
extern crate sdl2;
extern crate cgmath;
extern crate bytemuck;
extern crate clap;

mod cli;
mod bruker;
mod aht;
mod render;

use bruker::BrukerData;

fn main() {
    let cli_data = cli::InputData::from_cli();

    let bruker_data = BrukerData::from_csv(&cli_data.bruker_path);
    let aht_curve = aht::TorusSegment::aht_from_bruker(bruker_data, cli_data.delta_t, cli_data.amplitude);

    render::render(aht_curve, cli_data.resolution, &cli_data.shader);
}
