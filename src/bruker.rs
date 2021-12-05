use std::fs;
pub struct BrukerData {
    pub pulse_sequence: Vec<Vec<f64>>,
}
impl BrukerData {
    // .bruker files in CSV format (u_eff_trans/amplitude in %, phase phi)
    pub fn from_csv(path: &str) -> BrukerData {
        let contents = fs::read_to_string(path)
            .expect(&format!("File error: failed to read file {}", path))
            .replace(" ", "")
            .replace("\t", "");
        let mut matrix: Vec<Vec<f64>> = Vec::new();

        let mut line_count = 1;
        for line in contents.lines() {
            if &line[0..1] != "#" {
                let mut matrix_line: Vec<f64> = Vec::new();
                for substring in line.split(",") {
                    matrix_line.push(substring.parse::<f64>()
                                    .expect(&format!(".bruker parsing error: failed to parse line {}", line_count)));
                }
                matrix.push(matrix_line);
            }
            line_count += 1;
        }
        BrukerData { pulse_sequence: matrix }
    }
    pub fn _print_pulse(&self) {
        for i in self.pulse_sequence.iter() {
            println!("{:?}", i);
        }
    }
}
