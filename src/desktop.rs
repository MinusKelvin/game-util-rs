use glutin::window::WindowBuilder;
use glutin::event_loop::EventLoop;
use glutin::*;
use crate::prelude::*;

pub fn create_context<E>(
    wb: WindowBuilder,
    multisampling: u16,
    vsync: bool,
    el: &mut EventLoop<E>
) -> Result<(WindowedContext<PossiblyCurrent>, Gl), Box<dyn std::error::Error>> {
    let context = ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .with_gl_profile(GlProfile::Core)
        .with_multisampling(multisampling)
        .with_vsync(vsync)
        .build_windowed(wb, el)?;
    let context = unsafe { context.make_current() }.map_err(|(_, e)| e)?;
    let gl = std::rc::Rc::new(unsafe {
        glow::Context::from_loader_function(|s| context.get_proc_address(s))
    });

    unsafe {
        let vao = gl.create_vertex_array().unwrap();
        gl.bind_vertex_array(Some(vao));
    }

    Ok((context, gl))
}
