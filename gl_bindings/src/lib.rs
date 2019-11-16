use std::ops::Deref;
use std::rc::Rc;

#[allow(clippy::all)]
pub mod gl {
    // Include the generated OpenGL bindings
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

#[derive(Clone)]
pub struct Gl {
    inner: Rc<gl::Gl>,
}

impl Gl {
    pub fn load_with<F>(loadfn: F) -> Self
    where
        F: FnMut(&'static str) -> *const gl::types::GLvoid,
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
