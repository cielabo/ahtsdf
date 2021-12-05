# ahtsdf

Real-time rendering of AHT curves from .bruker files


<img src="https://user-images.githubusercontent.com/63974030/144765334-635123c6-20bb-45db-8fa5-4b7e14bbaa2b.png" alt="img text" style="width:250px;"/> <img src="https://user-images.githubusercontent.com/63974030/144765437-c74e2937-2dee-4322-b52e-eee736f88bd3.png" alt="img text" style="width:250px;"/> <img src="https://user-images.githubusercontent.com/63974030/144765814-7498ed51-8d24-4186-aa68-cd32cec449c9.png" alt="img text" style="width:250px;"/>

## dependencies
- The Rust programming language: [download](https://www.rust-lang.org/learn/get-started)
- The SDL2 development library: [download](https://www.libsdl.org/download-2.0.php)
- Hardware that supports OpenGL version 4.3 or newer.

SDL2 is a bit harder to set up on Windows: After unzipping the tarball, the DLL `SDL2-x.y.z/<platform_name>/bin/SDL2.dll` must be added to Rust's linker search path by moving it to `<rust_install_directory>/bin/rustlib/<build_target>/lib`.

## compiling
In the project's root directory `ahtsdf`, run `cargo build --release`.
Navigate to `target/release` and copy the binary `ahtsdf` into `ahtsdf/build`.

Alternatively, building and moving the binary can be automated by running `setup.sh`.

## how to run
From the `build` directory, run `./ahtsdf --help`. This will print a complete list of flags and options. The path argument (absolute or relative), specified by `-p` or `--path` is mandatory.
Running the program might look something like this:

`./ahtsdf -p ~/Documents/pulses/pulse015.bruker -d 50 -r 700 400 -s RAD_VEC`

In this case, the curve is rendered for a delta time value of 50 microseconds. It uses the shader `RAD_VEC` to display the image on a 700 by 400 pixel window. The `amplitude` parameter is not explicitly set, therefore its default value of 10000 Hz is used.

## installation
To avoid always having to run **ahtsdf** from its `build` directory, Linux users can copy its binary into `/usr/local/bin/` for immediate access regardless of working directory.
