#[macro_export]
macro_rules! implement_vert_component {
    ($impl_type:ty, $size:expr) => {
        impl VertComponent for $impl_type {
            fn attrib_pointer(gl: &Gl, location: u32, stride: usize, offset: i32) {
                unsafe {
                    gl.VertexAttribPointer(
                        location as crate::gl::types::GLuint,
                        $size,
                        crate::gl::FLOAT,
                        crate::gl::FALSE,
                        stride as crate::gl::types::GLint,
                        offset as *const crate::gl::types::GLvoid,
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
