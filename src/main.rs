#[macro_use]
extern crate render_derive;

use gl::types::{GLfloat, GLint, GLushort};
use gl_bindings::{gl, Gl};
use glfw::{
    Action, Context, Glfw, Key, OpenGlProfileHint, SwapInterval, Window, WindowEvent, WindowHint,
};
use nalgebra::{Matrix4, Orthographic3};
use render::{Index, Mesh, Shader, ShaderProgram, Uniform, Vec3, VertexAttrib};
use std::ffi::CString;
use std::ops::Deref;
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

#[derive(Debug, Copy, Clone)]
struct Mat4(Matrix4<f32>);

impl Deref for Mat4 {
    type Target = Matrix4<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Uniform for Mat4 {
    fn set_uniform(&self, gl: &Gl, location: GLint) {
        unsafe {
            gl.UniformMatrix4fv(location, 1, gl::FALSE, self.as_ptr() as *const GLfloat);
        }
    }
}

struct App<V: VertexAttrib, I: Index> {
    // Window
    glfw: Glfw,
    window: Window,
    events: Receiver<(f64, WindowEvent)>,

    // Draw testing
    shader: ShaderProgram,
    mesh: Mesh<V, I>,

    // Loop testing
    last_print_time: SystemTime,
    frames: i32,
    dir: bool,
    red: f32,

    // GL handle
    gl: Gl,
}

impl App<Vertex, GLushort> {
    fn new() -> Self {
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

        Self {
            glfw,
            window,
            events,

            shader: Self::init_test_shaders(&gl),
            mesh: Self::init_test_mesh(&gl),

            last_print_time: SystemTime::now(),
            frames: 0,
            dir: false,
            red: 0.0,

            gl,
        }
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
        let uniforms = vec!["projection_matrix", "red"]
            .into_iter()
            .map(|s| s.to_owned())
            .collect();
        ShaderProgram::new_from_shaders(&gl, vec![vert_shader, frag_shader], uniforms)
    }

    fn init_test_shaders(gl: &Gl) -> ShaderProgram {
        let vert_shader = include_str!("shader/basic_vertex.glsl");
        let frag_shader = include_str!("shader/basic_fragment.glsl");

        Self::create_shader_program(gl, vert_shader, frag_shader).unwrap()
    }

    fn init_test_mesh(gl: &Gl) -> Mesh<Vertex, GLushort> {
        let vertex_data: Vec<Vertex> = vec![
            // Bottom left
            Vertex::new((-1.0, -1.0, -0.5).into(), (1.0, 0.0, 0.0).into()),
            // Top left
            Vertex::new((-1.0, 1.0, -0.5).into(), (1.0, 1.0, 0.0).into()),
            // Top right
            Vertex::new((1.0, 1.0, -0.5).into(), (0.0, 1.0, 0.0).into()),
            // Bottom right
            Vertex::new((1.0, -1.0, -0.5).into(), (0.0, 0.0, 1.0).into()),
        ];
        let index_data: Vec<GLushort> = vec![0, 1, 2, 0, 2, 3];

        Mesh::create(gl, vertex_data, index_data)
    }

    fn start_game(&mut self) {
        // Keep looping until the user tries to close the window
        while !self.window.should_close() {
            // Poll for new events and handle all of them
            self.handle_window_events();

            // Handle a single loop tick
            self.loop_tick();
        }
    }

    fn loop_tick(&mut self) {
        // Clear the screen
        unsafe {
            self.gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        // Uniform testing
        if self.dir {
            self.red += 1.0f32 / 120.0f32;
            if self.red >= 1.0f32 {
                self.dir = !self.dir;
            }
        } else {
            self.red -= 1.0f32 / 120.0f32;
            if self.red <= 0.0f32 {
                self.dir = !self.dir;
            }
        }

        // Update projection matrix
        let win_size = self.window.get_size();
        let aspect = win_size.0 as f32 / win_size.1 as f32;
        let projection_ortho =
            Orthographic3::<f32>::new(aspect * -1.5, aspect * 1.5, -1.5, 1.5, 0.1, 1.0);

        // Draw a triangle
        self.shader.bind();
        self.shader.set_uniform("red", &self.red);
        self.shader
            .set_uniform("projection_matrix", &Mat4(*projection_ortho.as_matrix()));
        self.mesh.render();
        self.shader.unbind();

        // Display changes in the window
        self.window.swap_buffers();

        // Update frame counter
        let current_fps = self.update_frame_counter();
        if current_fps >= 0 {
            self.window
                .set_title(&format!("Citey | FPS: {}", current_fps));
        }
    }

    fn handle_window_events(&mut self) {
        // Tell GLFW to get the new events
        self.glfw.poll_events();

        // We have to put the events into a separate vec because `flush_messages`
        // uses an immutable borrow of `app` but the `handle_window_event` function
        // requires a mutable reference.
        let e: Vec<(f64, glfw::WindowEvent)> = glfw::flush_messages(&self.events).collect();

        // Loop through all of the captured events and try to handle them
        for (_, event) in e {
            self.handle_window_event(&event);
        }
    }

    fn handle_window_event(&mut self, event: &glfw::WindowEvent) {
        match event {
            glfw::WindowEvent::FramebufferSize(w, h) => unsafe {
                self.gl.Viewport(0, 0, *w, *h);

                // Keep drawing even while the window is being resized. In GLFW,
                // when resizing a window, the poll events call will handle until
                // the resizing is finished.
                // I commented this out because this would have to be called in the
                // unbuffered `poll_events` call, but this call requires funky
                // borrows so I have chosen not to implement it yet
                // self.loop_tick();
            },
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                self.window.set_should_close(true)
            }
            _ => {}
        }
    }

    fn update_frame_counter(&mut self) -> i32 {
        let current_loop_time = SystemTime::now();
        if current_loop_time
            .duration_since(self.last_print_time)
            .unwrap()
            .as_secs()
            >= 1
        {
            let val = self.frames;
            self.frames = 0;
            self.last_print_time = current_loop_time;
            return val;
        } else {
            self.frames += 1;
        }

        -1
    }
}

fn main() {
    let mut app = App::new();

    // Enable V-Sync
    app.glfw.set_swap_interval(SwapInterval::Sync(1));

    // Set the background color
    unsafe { app.gl.ClearColor(0.5f32, 0.5f32, 0.5f32, 1.0f32) };

    // Start the game loop
    app.start_game();
}
