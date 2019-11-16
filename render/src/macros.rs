#[macro_export]
macro_rules! implement_vert_component {
    ($impl_type:ty, $size:expr, $type:expr) => {
        impl VertComponent for $impl_type {
            fn attrib_pointer(gl: &Gl, location: u32, stride: usize, offset: i32) {
                unsafe {
                    gl.VertexAttribPointer(
                        location as gl_bindings::gl::types::GLuint,
                        $size,
                        $type,
                        gl_bindings::gl::FALSE,
                        stride as gl_bindings::gl::types::GLint,
                        offset as *const gl_bindings::gl::types::GLvoid,
                    );
                }
            }
        }
    };
}

#[macro_export]
macro_rules! implement_index {
    ($for_type:ty, $gl_type:expr) => {
        impl Index for $for_type {
            fn get_type() -> GLenum {
                $gl_type
            }
        }
    };
}
