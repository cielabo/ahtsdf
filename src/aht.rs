use cgmath::{Vector3, Matrix3, InnerSpace};
use super::bruker::BrukerData;
use std::f64::consts::PI;

#[derive(Debug)]
pub struct TorusSegment {
    pub normal: [f32; 3], // normal of circle surface the torus encapsulates
    pub center: [f32; 3],
    pub radius_vec: [f32; 3], // vector center -> beginning of torus segment
    pub angle: f32, // fraction of torus, full torus = PI (NOT 2*PI -> mirrored half torus)
}
impl TorusSegment {
    // returns a vector of torus segment data from BrukerData
    pub fn aht_from_bruker(data: BrukerData, delta_t: f64, amplitude: f64) -> Vec<TorusSegment> {

        let mut segments: Vec<TorusSegment> = Vec::new();
        let rad_scale = amplitude;

        let mut point = Vector3::new(0f64, 0f64, 0f64); // point where two torus segments connect
        let mut tangent = Vector3::new(0f64, 0f64, 1f64); // tangent to torus at point                        
        let mut normal = Vector3::new(-1f64, 0f64, 0f64);
        
        let mut prev_phi: f64 = 0.0; // phase of previous pulse

        for i in data.pulse_sequence.iter() {
            // i[0] = first column of .bruker data = (transversal part of field / amplitude) * 100
            // i[1] = second column = phi
            let u_x = (i[0]/100.0) * amplitude * i[1].to_radians().cos();
            let u_y = (i[0]/100.0) * amplitude * i[1].to_radians().sin();

            let u_eff = (u_x*u_x + u_y*u_y).sqrt();
            let alpha = 2.*PI * u_eff * delta_t; // angle that torus segment covers

            normal/*k*/ = (rotation_matrix(tangent/*k-1*/, (i[1].to_radians())-prev_phi) * normal/*k-1*/).normalize();

            let radius_vec = (rad_scale/u_eff)* tangent/*k-1*/.cross(normal/*k*/); // direction point -> center of torus segment
            let center = point/*k-1*/ + radius_vec; // center of torus segment

            let prev_point = point/*k-1*/;
            point = (rotation_matrix(normal/*k*/, alpha) * (prev_point/*k-1*/ - center/*k*/)) + center/*k*/;

            tangent = normal.cross((center/*k*/ - point/*k*/).normalize()); // faster than version 2 below
            //tangent = rotation_matrix(normal, alpha) * tangent; // version 2 of tangent calculation

            segments.push(TorusSegment{
                normal: as_cgvector_f32(normal.xzy()).into(),
                center: as_cgvector_f32(center.xzy()).into(),
                radius_vec: as_cgvector_f32(radius_vec.xzy()).into(),
                angle: (alpha / 2.0) as f32,

            });
            prev_phi = i[1].to_radians();
        }
        segments // return: list of torus segments from pulse data
    }
}
// unused, z is swapped in shader where necessary
fn _invert_z(v: Vector3<f64>) -> Vector3<f64>{
    Vector3::new(v.x, v.y, -v.z)
}
// returns a rotation matrix for rotation around axis a
fn rotation_matrix(a: Vector3<f64>, angle: f64) -> Matrix3<f64> {
    let s = angle.sin();
    let c = angle.cos();
    let ic = 1.0 - c;
    
    Matrix3::new(
        a.x*a.x*ic + c,       a.y*a.x*ic - s*a.z,    a.z*a.x*ic + s*a.y,
        a.x*a.y*ic + s*a.z,   a.y*a.y*ic + c,        a.z*a.y*ic - s*a.x,
        a.x*a.z*ic - s*a.y,   a.y*a.z*ic + s*a.x,    a.z*a.z*ic + c 
    )   
}
// converts CgVector of 64-bit floats to one of 32-bit floats
fn as_cgvector_f32(v: Vector3<f64>) -> Vector3<f32> {
    Vector3::new(v.x as f32, v.y as f32, v.z as f32)
}
