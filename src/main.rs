// Copyright 2015 Brendan Zabarauskas and the gl-rs developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate gl;
extern crate glutin;

use gl::types::*;
use std::mem;
use std::ptr;
use std::str;
use std::ffi::CString;
use std::os::raw::c_void;

#[cfg(target_os = "emscripten")]
pub mod emscripten;

// Vertex data (2*3*6 + 3*3*6)
static VERTEX_DATA: [GLfloat; 90] = [ 
    0.1,0.7,   0.1,0.2,0.3,   0.0,0.0,   0.1,0.2,0.3,  0.4,0.5,   0.1,0.2,0.3,
    0.5,0.4,   0.3,0.2,0.3,  0.0,0.0,   0.3,0.2,0.3,   0.5,-0.4,     0.3,0.2,0.3,
    0.4,-0.5,  0.4,0.2,0.3,   0.0,0.0, 0.4,0.2,0.3,   0.1,-0.7,     0.4,0.2,0.3,
    -0.1,-0.7,  0.5,0.2,0.3,  0.0,0.0,  0.5,0.2,0.3,  -0.4,-0.5,    0.5,0.2,0.3,
    -0.5,-0.4,   0.6,0.2,0.3,  0.0,0.0,  0.6,0.2,0.3, -0.5,0.4,   0.6,0.2,0.3,
    -0.4,0.5,   0.7,0.2,0.3,   0.0,0.0,  0.7,0.2,0.3, -0.1,0.7,     0.7,0.2,0.3,
];

static mut iterationcount: f32 = 0.0;
// Shader sources
static VS_SRC: &'static str = "#version 300 es
    layout (location = 0) in vec2 position;
    layout (location = 1) in vec3 aColor;
    out vec3 color;

    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
        color = aColor;
    }"
;
//out_color = vec4(gl_FragCoord.yxz / 320.0, 1.0);
//out_color = vec4(color, 1.0);
static FS_SRC: &'static str = "#version 300 es
    precision mediump float;
    in vec3 color;
    out vec4 out_color;

    void main() {
        
        out_color = vec4(color, 1.0);

    }"
;

fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    let shader;
    unsafe {
        shader = gl::CreateShader(ty);
        // Attempt to compile the shader
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Get the compile status
        let mut status = gl::FALSE as GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(
                shader,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );
            panic!(
                "{}",
                str::from_utf8(&buf)
                    .ok()
                    .expect("ShaderInfoLog not valid utf8")
            );
        }
    }
    shader
}

fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);
        // Get the link status
        let mut status = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        // Fail on error
        if status != (gl::TRUE as GLint) {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetProgramInfoLog(
                program,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );
            panic!(
                "{}",
                str::from_utf8(&buf)
                    .ok()
                    .expect("ProgramInfoLog not valid utf8")
            );
        }
        program
    }
}

fn main() {
    use glutin::GlContext;

    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new();
    let (api, version) = if cfg!(target_os = "emscripten") {
        (glutin::Api::WebGl, (2, 0))
    } else {
        (glutin::Api::OpenGlEs, (3, 0))
    };
    let context = glutin::ContextBuilder::new()
        .with_gl(glutin::GlRequest::Specific(api, version));
    let gl_window = glutin::GlWindow::new(window, context, &events_loop)
        .unwrap();
    unsafe { gl_window.set_title("Nalin1o1 - 6  triangle Flower")};

    // It is essential to make the context current before calling `gl::load_with`.
    unsafe { gl_window.make_current() }.unwrap();

    // Load the OpenGL function pointers
    // TODO: `as *const _` will not be needed once glutin is updated to the latest gl version
    gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);

    // Create GLSL shaders
    let vs = compile_shader(VS_SRC, gl::VERTEX_SHADER);
    let fs = compile_shader(FS_SRC, gl::FRAGMENT_SHADER);
    let program = link_program(vs, fs);

    let mut vao = 0;
    let mut vbo = 0;
    let mut vbocolor = 0;

    println!("Program linked");

    unsafe {
        // Create Vertex Array Object
        if !cfg!(target_os = "emscripten") {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
        }

        // Create a Vertex Buffer Object and copy the vertex data to it
        gl::GenBuffers(1, &mut vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (VERTEX_DATA.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            mem::transmute(&VERTEX_DATA[0]),
            gl::STATIC_DRAW,
        );

        println!("Data uploaded");

        // Use shader program
        gl::UseProgram(program);

        // Specify the layout of the vertex data - for position
        let pos_attr = gl::GetAttribLocation(program, CString::new("position").unwrap().as_ptr());
        println!("pos attrib {}", pos_attr);
        gl::EnableVertexAttribArray(pos_attr as GLuint);
        gl::VertexAttribPointer(
            pos_attr as GLuint,
            2,
            gl::FLOAT,
            gl::FALSE as GLboolean,
            5 * mem::size_of::<GLfloat>() as GLsizei,
            ptr::null(),
        );

        // VBO for color
        
        let col_attr = gl::GetAttribLocation(program, CString::new("aColor").unwrap().as_ptr());
        println!("color attrib {}", col_attr);
        gl::EnableVertexAttribArray(col_attr  as GLuint);
        gl::VertexAttribPointer(
            col_attr as GLuint,
            3,
            gl::FLOAT,
            gl::FALSE as GLboolean,
             5 * mem::size_of::<GLfloat>() as GLsizei,
            (2 * mem::size_of::<GLfloat>()) as *const c_void
        );
        
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
        println!("Data specified");
    }

    // Events get generated only when mouse moves over th window
    let draw_iter = || {

        unsafe {

            iterationcount = iterationcount + 0.001;
            if iterationcount > 0.9
            {
                iterationcount = 0.0;
            }
            // Clear the screen to black
            gl::ClearColor(iterationcount, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Draw 6 triangles from the 18 vertices
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 18);
        }

        gl_window.swap_buffers().unwrap();
    };

    #[cfg(target_os = "emscripten")]
    emscripten::set_main_loop_callback(draw_iter);

    #[cfg(not(target_os = "emscripten"))]
    events_loop.run_forever(|event| {
        use glutin::{ControlFlow, Event, WindowEvent};

        if let Event::WindowEvent { event, .. } = event {
            if let WindowEvent::Closed = event {
                return ControlFlow::Break;
            }
        }

        draw_iter();

        ControlFlow::Continue
    });

    // Cleanup
    unsafe {
        gl::DeleteProgram(program);
        gl::DeleteShader(fs);
        gl::DeleteShader(vs);
        gl::DeleteBuffers(1, &vbo);
        if !cfg!(target_os = "emscripten") {
            gl::DeleteVertexArrays(1, &vao);
        }
    }
}