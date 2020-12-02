use glutin::window::WindowBuilder;
use glutin::event_loop::EventLoop;
use glutin::*;
use glutin::dpi;

pub fn create_context<E>(
    wb: WindowBuilder,
    multisampling: u16,
    vsync: bool,
    el: &mut EventLoop<E>
) -> Result<WindowedContext<PossiblyCurrent>, Box<dyn std::error::Error>> {
    let context = ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .with_gl_profile(GlProfile::Core)
        .with_multisampling(multisampling)
        .with_vsync(vsync)
        .build_windowed(wb, el)?;
    let context = unsafe { context.make_current() }.map_err(|(_, e)| e)?;
    gl::load_with(|s| context.get_proc_address(s) as *const _);

    unsafe {
        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
    }

    Ok(context)
}

pub fn clamp_aspect(lsize: dpi::LogicalSize<f64>) -> dpi::LogicalSize<f64> {
    let ratio = lsize.width / lsize.height;
    if ratio > 16.0 / 8.0 {
        let ratio = 16.0 / 8.0;
        let w = lsize.height * ratio;
        dpi::LogicalSize::new(w, lsize.height)
    } else if ratio < 16.0 / 10.0 {
        let ratio = 16.0 / 10.0;
        let h = lsize.width / ratio;
        dpi::LogicalSize::new(lsize.width, h)
    } else {
        lsize
    }
}