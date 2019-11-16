#[macro_use]
extern crate render_derive;

use gl::types::GLushort;
use gl_bindings::{gl, Gl};
use glfw::{
    Action, Context, Glfw, Key, OpenGlProfileHint, SwapInterval, Window, WindowEvent, WindowHint,
};
use render::{Mesh, Shader, ShaderProgram, Vec3};
use std::ffi::CString;
use std::sync::mpsc::Receiver;
use std::time::SystemTime;

#[derive(VertexAttribPointers, Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct Vertex {
    #[location = 0]
    pub pos: Vec3,

    #[location = 1]
    pub col: Vec3,
}

impl Vertex {
    pub fn new(pos: Vec3, col: Vec3) -> Self {
        Self { pos, col }
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

    glfw.with_primary_monitor_mut(|_, m| {
        if let Some(monitor) = m {
            // Get the monitor information
            let monitor_vidmode = monitor
                .get_video_mode()
                .expect("failed to get monitor vidmode");

            // Set the window size to 2/3 of the monitor size
            window.set_size(
                monitor_vidmode.width as i32 * 2 / 3,
                monitor_vidmode.height as i32 * 2 / 3,
            );

            // Center the monitor on the screen
            window.set_pos(
                (monitor_vidmode.width as i32 - window.get_size().0) / 2,
                (monitor_vidmode.height as i32 - window.get_size().1) / 2,
            );
        }
    });

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
    let index_data: Vec<GLushort> = vec![0, 1, 2, 0, 2, 3];
    let mesh = Mesh::create(&gl, vertex_data, index_data);

    let mut red = 0.0f32;
    let mut dir = false;

    // Keep looping until the user tries to close the window
    while !window.should_close() {
        // Poll for new events and handle all of them
        handle_window_events(&gl, &mut glfw, &events, &mut window);

        // Clear the screen
        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // Uniform test
        if dir {
            red += 1.0f32 / 120.0f32;
            if red >= 1.0f32 {
                dir = !dir;
            }
        } else {
            red -= 1.0f32 / 120.0f32;
            if red <= 0.0f32 {
                dir = !dir;
            }
        }

        // Draw a triangle
        shader.bind();
        shader.set_uniform("red", red);
        mesh.render();
        shader.unbind();

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
        glfw::WindowEvent::FramebufferSize(w, h) => unsafe { gl.Viewport(0, 0, w, h) },
        glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
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
    let vert_shader = include_str!("shader/basic_vertex.glsl");
    let frag_shader = include_str!("shader/basic_fragment.glsl");

    create_shader_program(gl, vert_shader, frag_shader).unwrap()
}

fn create_shader_program(
    gl: &Gl,
    vertex_shader: &str,
    fragment_shader: &str,
) -> Result<ShaderProgram, String> {
    let vert_shader = Shader::new_from_source(
        &gl,
        gl::VERTEX_SHADER,
        &CString::new(vertex_shader).unwrap(),
    )?;

    let frag_shader = Shader::new_from_source(
        &gl,
        gl::FRAGMENT_SHADER,
        &CString::new(fragment_shader).unwrap(),
    )?;

    // Uniforms are defined when the shader program is created to prevent the
    // slowdown possibly incurred by getting the location of a shader at
    // runtime
    let uniforms = vec!["red".to_owned()];
    ShaderProgram::new_from_shaders(&gl, vec![vert_shader, frag_shader], uniforms)
}
