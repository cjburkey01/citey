#[macro_use]
extern crate render_derive;

use crate::render::{Vertex, VertexArray, VertexAttrib};
use gl::types::{GLushort, GLvoid};
use glfw::{
    Action, Context, Glfw, Key, OpenGlProfileHint, SwapInterval, Window, WindowEvent, WindowHint,
};
use render::{Buffer, Shader, ShaderProgram};
use std::ffi::CString;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::mpsc::Receiver;
use std::time::SystemTime;

#[allow(clippy::all)]
pub mod gl {
    // Include the generated OpenGL bindings
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

pub mod render;

#[derive(Clone)]
pub struct Gl {
    inner: Rc<gl::Gl>,
}

impl Gl {
    pub fn load_with<F>(loadfn: F) -> Self
    where
        F: FnMut(&'static str) -> *const GLvoid,
    {
        Self {
            inner: Rc::new(gl::Gl::load_with(loadfn)),
        }
    }
}

impl Deref for Gl {
    type Target = gl::Gl;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

fn main() {
    println!("Hello, world!");

    // Initialize GLFW
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).expect("failed to initialize GLFW");

    // Initialize OpenGL window hints
    // Make sure we tell the window to use OpenGL 3.3 Core (and forward
    // compatible)
    glfw.window_hint(WindowHint::ContextVersion(3, 3));
    glfw.window_hint(WindowHint::OpenGlForwardCompat(true));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));

    // Create the window and the events system
    let (mut window, events) = glfw
        .create_window(300, 300, "Window", glfw::WindowMode::Windowed)
        .expect("failed to create GLFW window");
    window.set_all_polling(true);

    // Make the window's context current
    window.make_current();

    // Tell OpenGL how to access methods and get an instance of the Gl struct
    let gl = Gl::load_with(|s| window.get_proc_address(s) as *const _);

    // Enable V-Sync
    glfw.set_swap_interval(SwapInterval::Sync(1));

    // Set the background color
    unsafe {
        gl.ClearColor(0.5f32, 0.5f32, 0.5f32, 1.0f32);
    }

    // Start the game loop
    start_game(glfw, window, events, gl);
}

fn start_game(mut glfw: Glfw, mut window: Window, events: Receiver<(f64, WindowEvent)>, gl: Gl) {
    let mut last_print_time = SystemTime::now();
    let mut frames = 0;

    let shader = init_shaders(&gl);

    let vao = VertexArray::new(&gl);
    vao.bind();

    let mut vbo = Buffer::new(&gl);
    let vertex_data: Vec<Vertex> = vec![
        // Bottom left
        Vertex::new((-0.5, -0.5, 0.0).into(), (1.0, 0.0, 0.0).into()),
        // Top left
        Vertex::new((-0.5, 0.5, 0.0).into(), (1.0, 1.0, 0.0).into()),
        // Top right
        Vertex::new((0.5, 0.5, 0.0).into(), (0.0, 1.0, 0.0).into()),
        // Bottom right
        Vertex::new((0.5, -0.5, 0.0).into(), (0.0, 0.0, 1.0).into()),
    ];
    vbo.buffer(gl::ARRAY_BUFFER, gl::STATIC_DRAW, vertex_data);
    vbo.bind(gl::ARRAY_BUFFER);
    Vertex::setup_attrib_pointer(&gl);
    vbo.unbind(gl::ARRAY_BUFFER);

    let mut ebo = Buffer::new(&gl);
    let element_data: Vec<GLushort> = vec![0, 1, 2, 0, 2, 3];
    ebo.buffer(gl::ELEMENT_ARRAY_BUFFER, gl::STATIC_DRAW, element_data);

    vao.unbind();

    // Keep looping until the user tries to close the window
    while !window.should_close() {
        // Poll for new events and handle all of them
        handle_window_events(&gl, &mut glfw, &events, &mut window);

        // Clear the screen
        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // Draw a triangle
        {
            shader.bind();
            vao.bind();
            ebo.bind(gl::ELEMENT_ARRAY_BUFFER);
            unsafe { gl.EnableVertexAttribArray(0) };
            unsafe { gl.EnableVertexAttribArray(1) };
            unsafe { gl.DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_SHORT, std::ptr::null()) };
            unsafe { gl.DisableVertexAttribArray(1) };
            unsafe { gl.DisableVertexAttribArray(0) };
            ebo.unbind(gl::ELEMENT_ARRAY_BUFFER);
            vao.unbind();
            shader.unbind();
        }

        // Display changes in the window
        window.swap_buffers();

        // Update frame counter
        let current_fps = update_frame_counter(&mut last_print_time, &mut frames);
        if current_fps >= 0 {
            window.set_title(&format!("Citey | FPS: {}", current_fps));
        }
    }
}

fn handle_window_events(
    gl: &Gl,
    glfw: &mut glfw::Glfw,
    events: &Receiver<(f64, WindowEvent)>,
    window: &mut glfw::Window,
) {
    glfw.poll_events();
    for (_, event) in glfw::flush_messages(events) {
        handle_window_event(gl, window, event);
    }
}

fn handle_window_event(gl: &Gl, window: &mut glfw::Window, event: glfw::WindowEvent) {
    match event {
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
        glfw::WindowEvent::FramebufferSize(w, h) => unsafe { gl.Viewport(0, 0, w, h) },
        _ => {}
    }
}

fn update_frame_counter(last_print_time: &mut SystemTime, frames: &mut i32) -> i32 {
    let current_loop_time = SystemTime::now();
    if current_loop_time
        .duration_since(*last_print_time)
        .unwrap()
        .as_secs()
        >= 1
    {
        let val = *frames;
        *frames = 0;
        *last_print_time = current_loop_time;
        return val;
    } else {
        *frames += 1;
    }

    -1
}

fn init_shaders(gl: &Gl) -> ShaderProgram {
    const VERT_SHADER: &str = include_str!("shader/basic_vertex.glsl");
    const FRAG_SHADER: &str = include_str!("shader/basic_fragment.glsl");

    let vert_shader =
        Shader::new_from_source(&gl, gl::VERTEX_SHADER, &CString::new(VERT_SHADER).unwrap())
            .expect("failed to compile vertex shader");
    let frag_shader = Shader::new_from_source(
        &gl,
        gl::FRAGMENT_SHADER,
        &CString::new(FRAG_SHADER).unwrap(),
    )
    .expect("failed to compile fragment shader");

    ShaderProgram::new_from_shaders(&gl, vec![vert_shader, frag_shader])
        .expect("failed to link shader program")
}
