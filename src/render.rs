use crate::gl::types::{GLchar, GLenum, GLint, GLsizeiptr, GLuint, GLushort, GLvoid};
use crate::Gl;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::mem::size_of;

fn create_empty_vec_cstr(len: usize) -> Vec<u8> {
    // Create a vec with enough capacity for the string
    let mut info_log_raw: Vec<u8> = Vec::with_capacity(len + 1);

    // Fill the vec with spaces except for the last character, which should be
    // null
    info_log_raw.extend([b' '].iter().cycle().take(len as usize));

    // Return the vec
    info_log_raw
}

fn create_str_from_raw_cstr(mut str: Vec<u8>) -> String {
    // If the string ends with a null character, remove it
    if str[str.len() - 1] == b'\0' {
        str.pop();
    }

    // Convert the raw bytes into a String
    unsafe { String::from_utf8_unchecked(str) }
}

pub struct Shader {
    id: GLuint,
    gl: Gl,
}

impl Shader {
    fn new(gl: &Gl, shader_type: GLenum) -> Self {
        Self {
            id: unsafe { gl.CreateShader(shader_type) },
            gl: gl.clone(),
        }
    }

    pub fn new_from_source(gl: &Gl, shader_type: GLenum, source: &CStr) -> Result<Self, String> {
        let shader = Self::new(gl, shader_type);

        // Load the source into the shader and attempt to compile it
        unsafe {
            gl.ShaderSource(shader.id, 1, &source.as_ptr(), std::ptr::null());
            gl.CompileShader(shader.id);
        }

        // Check for shader compilation errors
        if let Err(err) = shader.check_compile_error() {
            return Err(err);
        }

        // Return the shader
        Ok(shader)
    }

    fn check_compile_error(&self) -> Result<(), String> {
        // Get the length of the error log
        let mut info_log_length: GLint = 0;
        unsafe {
            self.gl
                .GetShaderiv(self.id, crate::gl::INFO_LOG_LENGTH, &mut info_log_length)
        };
        if info_log_length > 0 {
            // Load the error log into a vec of u8
            let mut info_log_raw = create_empty_vec_cstr(info_log_length as usize);
            unsafe {
                self.gl.GetShaderInfoLog(
                    self.id,
                    info_log_length,
                    std::ptr::null_mut(),
                    info_log_raw.as_mut_ptr() as *mut GLchar,
                )
            };

            // Turn the vec of u8 into a string to print into the console
            return Err(create_str_from_raw_cstr(info_log_raw));
        }

        Ok(())
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { self.gl.DeleteShader(self.id) };
        println!("Dropping shader {}", self.id);
    }
}

pub struct ShaderProgram {
    id: GLuint,
    gl: Gl,
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe { self.gl.DeleteProgram(self.id) };
        println!("Dropping shader program {}", self.id);
    }
}

impl ShaderProgram {
    fn new(gl: &Gl) -> Self {
        Self {
            id: unsafe { gl.CreateProgram() },
            gl: gl.clone(),
        }
    }

    pub fn new_from_shaders(gl: &Gl, shaders: Vec<Shader>) -> Result<Self, String> {
        let program = Self::new(gl);

        // Attach the shaders
        for shader in shaders.iter() {
            unsafe { gl.AttachShader(program.id, shader.id) };
        }

        // Link the program and check for any linking errors
        unsafe { gl.LinkProgram(program.id) };
        if let Err(err) = program.check_link_error() {
            return Err(err);
        }

        // Detach the shaders
        // When the shaders are dropped at the end of this method, they will be
        // registered as deleted and must be detached for that to happen
        for shader in shaders.into_iter() {
            unsafe { gl.DetachShader(program.id, shader.id) };
        }

        // Return the program
        Ok(program)
    }

    fn check_link_error(&self) -> Result<(), String> {
        // Get the length of the error log
        let mut info_log_length: GLint = 0;
        unsafe {
            self.gl
                .GetProgramiv(self.id, crate::gl::INFO_LOG_LENGTH, &mut info_log_length)
        };

        if info_log_length > 0 {
            // Load the error log into a vec of u8
            let mut info_log_raw = create_empty_vec_cstr(info_log_length as usize);
            unsafe {
                self.gl.GetProgramInfoLog(
                    self.id,
                    info_log_length,
                    std::ptr::null_mut(),
                    info_log_raw.as_mut_ptr() as *mut GLchar,
                )
            };

            // Turn the vec of u8 into a string to print into the console
            return Err(create_str_from_raw_cstr(info_log_raw));
        }

        Ok(())
    }

    pub fn bind(&self) {
        unsafe { self.gl.UseProgram(self.id) };
    }

    pub fn unbind_all(gl: &Gl) {
        unsafe { gl.UseProgram(0) };
    }

    pub fn unbind(&self) {
        Self::unbind_all(&self.gl);
    }
}

pub struct Buffer {
    id: GLuint,
    gl: Gl,
}

impl Buffer {
    pub fn new(gl: &Gl) -> Self {
        Self {
            id: {
                let mut buff: GLuint = 0;
                unsafe { gl.GenBuffers(1, &mut buff) };
                buff
            },
            gl: gl.clone(),
        }
    }

    pub fn bind(&self, location: GLenum) {
        unsafe { self.gl.BindBuffer(location, self.id) };
    }

    pub fn unbind_all(gl: &Gl, location: GLenum) {
        unsafe { gl.BindBuffer(location, 0) };
    }

    pub fn unbind(&self, location: GLenum) {
        Self::unbind_all(&self.gl, location);
    }

    fn buffer_raw(
        &self,
        location: GLenum,
        usage: GLenum,
        size: usize,
        data: *const GLvoid,
        bind: bool,
    ) {
        if bind {
            self.bind(location);
        }
        unsafe {
            self.gl
                .BufferData(location, size as GLsizeiptr, data, usage);
        };
        if bind {
            self.unbind(location);
        }
    }

    pub fn buffer<T>(&mut self, location: GLenum, usage: GLenum, data: Vec<T>, bind: bool) {
        self.buffer_raw(
            location,
            usage,
            data.len() * size_of::<T>(),
            data.as_ptr() as *const GLvoid,
            bind,
        );
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe { self.gl.DeleteBuffers(1, &self.id) };
        println!("Dropping buffer {}", self.id);
    }
}

pub struct VertexArray {
    id: GLuint,
    gl: Gl,
}

impl VertexArray {
    pub fn new(gl: &Gl) -> Self {
        Self {
            id: {
                let mut vao: GLuint = 0;
                unsafe { gl.GenVertexArrays(1, &mut vao) };
                vao
            },
            gl: gl.clone(),
        }
    }

    pub fn bind(&self) {
        unsafe { self.gl.BindVertexArray(self.id) };
    }

    pub fn unbind_all(gl: &Gl) {
        unsafe { gl.BindVertexArray(0) };
    }

    pub fn unbind(&self) {
        Self::unbind_all(&self.gl);
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe { self.gl.DeleteVertexArrays(1, &self.id) };
        println!("Dropping vertex array {}", self.id);
    }
}

pub struct Mesh<T: VertexAttrib> {
    vao: VertexArray,
    #[allow(dead_code)]
    vbo: Buffer,
    ebo: Buffer,
    indices: usize,
    gl: Gl,
    phantom: PhantomData<T>,
}

impl<T: VertexAttrib> Mesh<T> {
    fn new(vao: VertexArray, vbo: Buffer, ebo: Buffer, indices: usize, gl: &Gl) -> Self {
        Self {
            vao,
            vbo,
            ebo,
            indices,
            gl: gl.clone(),
            phantom: PhantomData,
        }
    }

    pub fn create(gl: &Gl, vertex_data: Vec<T>, index_data: Vec<GLushort>) -> Self {
        let vao = VertexArray::new(gl);
        vao.bind();

        let mut vbo = Buffer::new(&gl);
        vbo.bind(crate::gl::ARRAY_BUFFER);
        vbo.buffer(
            crate::gl::ARRAY_BUFFER,
            crate::gl::STATIC_DRAW,
            vertex_data,
            false,
        );
        T::setup_attrib_pointer(&gl);
        vbo.unbind(crate::gl::ARRAY_BUFFER);

        let index_count = index_data.len();
        let mut ebo = Buffer::new(&gl);
        ebo.buffer(
            crate::gl::ELEMENT_ARRAY_BUFFER,
            crate::gl::STATIC_DRAW,
            index_data,
            true,
        );

        vao.unbind();

        Self::new(vao, vbo, ebo, index_count, gl)
    }

    pub fn render(&self) {
        self.vao.bind();
        self.ebo.bind(crate::gl::ELEMENT_ARRAY_BUFFER);
        T::enable_attribs(&self.gl);
        unsafe {
            self.gl.DrawElements(
                crate::gl::TRIANGLES,
                self.indices as i32,
                crate::gl::UNSIGNED_SHORT,
                std::ptr::null(),
            )
        };
        T::disable_attribs(&self.gl);
        self.ebo.unbind(crate::gl::ELEMENT_ARRAY_BUFFER);
        self.vao.unbind();
    }
}

pub trait VertComponent {
    fn attrib_pointer(gl: &Gl, location: u32, stride: usize, offset: i32);
}

pub trait VertexAttrib {
    fn setup_attrib_pointer(gl: &Gl);

    fn enable_attribs(gl: &Gl);

    fn disable_attribs(gl: &Gl);
}

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl VertComponent for Vec3 {
    fn attrib_pointer(gl: &Gl, location: u32, stride: usize, offset: i32) {
        unsafe {
            gl.VertexAttribPointer(
                location as crate::gl::types::GLuint,
                3,
                crate::gl::FLOAT,
                crate::gl::FALSE,
                stride as crate::gl::types::GLint,
                offset as *const crate::gl::types::GLvoid,
            );
        }
    }
}

impl From<(f32, f32, f32)> for Vec3 {
    fn from(tuple: (f32, f32, f32)) -> Self {
        Self::new(tuple.0, tuple.1, tuple.2)
    }
}

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
