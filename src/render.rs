use glow::*;
use sdl2::event::Event;
use std::time::Instant;
use std::fs::read_to_string;
use bytemuck::cast_slice;
use crate::aht::TorusSegment;

pub fn render(torus_data: Vec<TorusSegment>, resolution: [u32; 2], shader: &str) {
    unsafe {
        // OpenGL context creation, SDL2 window setup
        let (gl, shader_version, window, mut event_pump, _context) = {
            let sdl = sdl2::init().unwrap();
            let video = sdl.video().unwrap();
            let gl_attr = video.gl_attr();
            gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
            gl_attr.set_context_version(4, 0);
            let window = video
                .window("ahtsdf", resolution[0], resolution[1])
                .opengl()
                .build()
                .unwrap();
            let context = window.gl_create_context().unwrap();
            let gl =
                glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _);
            let event_pump = sdl.event_pump().unwrap();
            (gl, "#version 430", window, event_pump, context)
        };

        // vao = Vertex Array Object
        let vao = gl
            .create_vertex_array()
            .expect("Render error: failed to create vertex array");
        gl.bind_vertex_array(Some(vao));

        // Program creation, shader compilation
        let program = gl.create_program().expect("Render error: failed to create program");

        // Vertex shader baked into Rust code due to its simplicity
        let vert_shader = 
            r#"
            const vec2 verts[6] = vec2[6](
                vec2(-1.0f,  1.0f),
                vec2(-1.0f, -1.0f),
                vec2( 1.0f, -1.0f),

                vec2(-1.0f,  1.0f),
                vec2( 1.0f, -1.0f),
                vec2( 1.0f,  1.0f)
            );
            void main() {
                vec2 pos = verts[gl_VertexID];
                gl_Position = vec4(pos, 0.0, 1.0);
            }
            "#;

        // code is appended to custom preprocessing constant definition, sort of shader metaprogramming 
        // (MAX_OBJECTS = number of torus segments, important for bounding volumes and loops)
        let mut define = String::from("#define MAX_OBJECTS ");
        let const_segments = format!("{}\n", torus_data.len());
        define.push_str(&const_segments);
        let code = read_to_string(shader).unwrap();
        define.push_str(&code);
        let frag_shader = &define;

        let shader_sources = [
            (glow::VERTEX_SHADER, vert_shader),
            (glow::FRAGMENT_SHADER, frag_shader),
        ];

        // creating and compiling shader objects
        let mut shaders = Vec::with_capacity(shader_sources.len());
        for (shader_type, shader_source) in shader_sources.iter() {
            let shader = gl
                .create_shader(*shader_type)
                .expect("Render error: failed to create shader");
            gl.shader_source(shader, &format!("{}\n{}", shader_version, shader_source));
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                panic!("Render error: failed to compile shader: {}", gl.get_shader_info_log(shader));
            }
            gl.attach_shader(program, shader);
            shaders.push(shader);
        }

        // OpenGL program linking
        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!("Render error: failed to link program: {}", gl.get_program_info_log(program));
        }

        // splitting Vec<TorusData> into multiple Vec<T> for the individual struct fields
        let mut torus_normals: Vec<[f32; 4]> = Vec::new();
        let mut torus_centers: Vec<[f32; 4]> = Vec::new();
        let mut torus_rad_vec: Vec<[f32; 4]> = Vec::new();
        let mut torus_angles: Vec<f32> = Vec::new();

        let mut min_x = 100000.0f32;
        let mut min_y = 100000.0f32;
        let mut max_z = 100000.0f32;

        // determining minimum coordinates of AHT curve, relevant for placement of planes in "DEFAULT" shader
        for i in torus_data.iter(){
            torus_normals.push([i.normal[0], i.normal[1], i.normal[2], 0.0]);
            torus_centers.push([i.center[0], i.center[1], i.center[2], 0.0]);
            torus_rad_vec.push([i.radius_vec[0], i.radius_vec[1], i.radius_vec[2], 0.0]);
            torus_angles.push(i.angle);
            let radius_len = (i.radius_vec[0]*i.radius_vec[0] + i.radius_vec[1]*i.radius_vec[1] + i.radius_vec[2]*i.radius_vec[2]).sqrt();
            min_x = min_x.min(i.center[0]-radius_len);
            min_y = min_y.min(i.center[1]-radius_len);
            max_z = max_z.min(i.center[2]-radius_len);

        };
        
        // transforming slices of Vec<T> into &[u8] for OpenGL buffer padding needs 
        let torus_normals: &[u8] = cast_slice(&torus_normals[..]);
        let torus_centers: &[u8] = cast_slice(&torus_centers[..]);
        let torus_rad_vec: &[u8] = cast_slice(&torus_rad_vec[..]);
        let torus_angles: &[u8] = cast_slice(&torus_angles[..]);

        // getting uniform locations freed by OpenGL when compiling the shaders
        let loc_time = gl.get_uniform_location(program, "u_time");
        let loc_res = gl.get_uniform_location(program, "u_resolution");
        let loc_mouse = gl.get_uniform_location(program, "u_mouse");
        let loc_num_segments = gl.get_uniform_location(program, "num_segments");
        let loc_planes = gl.get_uniform_location(program, "planes");

        // binding torus data buffers
        let normals = gl.create_buffer().unwrap();
        gl.bind_buffer(SHADER_STORAGE_BUFFER, Some(normals));
        gl.buffer_data_u8_slice(SHADER_STORAGE_BUFFER, torus_normals,  STATIC_DRAW);
        gl.bind_buffer_base(SHADER_STORAGE_BUFFER, 0, Some(normals));
        gl.bind_buffer(SHADER_STORAGE_BUFFER, Some(normals));

        let centers = gl.create_buffer().unwrap();
        gl.bind_buffer(SHADER_STORAGE_BUFFER, Some(centers));
        gl.buffer_data_u8_slice(SHADER_STORAGE_BUFFER, torus_centers,  STATIC_DRAW);
        gl.bind_buffer_base(SHADER_STORAGE_BUFFER, 1, Some(centers));
        gl.bind_buffer(SHADER_STORAGE_BUFFER, Some(centers));

        let rad_vec = gl.create_buffer().unwrap();
        gl.bind_buffer(SHADER_STORAGE_BUFFER, Some(rad_vec));
        gl.buffer_data_u8_slice(SHADER_STORAGE_BUFFER, torus_rad_vec,  STATIC_DRAW);
        gl.bind_buffer_base(SHADER_STORAGE_BUFFER, 2, Some(rad_vec));
        gl.bind_buffer(SHADER_STORAGE_BUFFER, Some(rad_vec));

        let angles = gl.create_buffer().unwrap();
        gl.bind_buffer(SHADER_STORAGE_BUFFER, Some(angles));
        gl.buffer_data_u8_slice(SHADER_STORAGE_BUFFER, torus_angles,  STATIC_DRAW);
        gl.bind_buffer_base(SHADER_STORAGE_BUFFER, 3, Some(angles));
        gl.bind_buffer(SHADER_STORAGE_BUFFER, Some(angles));

        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }

        gl.use_program(Some(program));
        gl.clear_color(1.0, 0.0, 1.0, 1.0);

        let mut running = true;

        let timer = Instant::now();
        let mut _elapsed = timer.elapsed().as_secs_f32();
        let mut mouse : [f32; 4] = [0.5*(resolution[0] as f32), 0.5*(resolution[1] as f32), 0.0, 0.0];

        // Update loop, runs once for each rendered frame
        while running {
            {
                // SDL2 window events, includes mouse movement and input as well as application quit
                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit { .. } => running = false,
                        Event::MouseButtonDown {mouse_btn, x, y, ..} => {
                            if let sdl2::mouse::MouseButton::Left = mouse_btn {
                                mouse = [x as f32, (-y as f32) + (resolution[1] as f32), 1.0, mouse[3]];
                            };
                        },
                        Event::MouseButtonUp {mouse_btn, ..} => {
                            if let sdl2::mouse::MouseButton::Left = mouse_btn {
                                mouse[2] = 0.0;
                            };
                        },
                        Event::MouseMotion {mousestate, x, y, ..} => {
                            if mousestate.left() {
                                mouse = [x as f32, (-y as f32) + (resolution[1] as f32), 1.0, mouse[3]];
                            }else{
                                mouse[2] = 0.0;
                            }
                        },
                        Event::MouseWheel {y, ..} => {
                            mouse[3] += 0.05 * (y.signum() as f32);
                            mouse[3] = mouse[3].clamp(-1.0, 1.0)
                        },
                        _ => {}
                    }
                }
            }

            gl.clear(COLOR_BUFFER_BIT);

            // uniform updates
            _elapsed = timer.elapsed().as_secs_f32();
            gl.uniform_1_f32(loc_time.as_ref(), _elapsed);
            gl.uniform_2_f32(loc_res.as_ref(), resolution[0] as f32, resolution[1] as f32);
            gl.uniform_4_f32(loc_mouse.as_ref(), mouse[0], mouse[1], mouse[2], mouse[3]);
            gl.uniform_1_u32(loc_num_segments.as_ref(), torus_data.len() as u32);
            gl.uniform_3_f32(loc_planes.as_ref(), min_x, min_y, max_z);

            // redraw
            gl.draw_arrays(glow::TRIANGLES, 0, 6);
            window.gl_swap_window();

            // exit
            if !running {
                gl.delete_program(program);
                gl.delete_vertex_array(vao);
            }
        }
    }
}