use glutin::*;

pub fn create_context(
    wb: WindowBuilder,
    multisampling: u16,
    vsync: bool,
    el: &mut EventsLoop
) -> (ContextWrapper<PossiblyCurrent, Window>, dpi::LogicalSize) {
    let context = ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .with_multisampling(multisampling)
        .with_vsync(vsync)
        .build_windowed(wb, el)
        .unwrap();
    let context = unsafe { context.make_current() }.unwrap();
    gl::load_with(|s| context.get_proc_address(s) as *const _);

    let mut lsize = None;
    el.poll_events(|event|
        if let Event::WindowEvent { event: WindowEvent::Resized(ls), .. } = event {
            lsize = Some(ls);
        }
    );

    unsafe {
        let mut vao = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);
    }

    (context, lsize.unwrap())
}

pub fn clamp_aspect(lsize: dpi::LogicalSize) -> dpi::LogicalSize {
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