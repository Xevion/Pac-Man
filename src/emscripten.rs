#[allow(dead_code)]
#[cfg(target_os = "emscripten")]
pub mod emscripten {
    use std::os::raw::c_uint;

    extern "C" {
        pub fn emscripten_get_now() -> f64;
        pub fn emscripten_sleep(ms: c_uint);
        pub fn emscripten_get_element_css_size(target: *const u8, width: *mut f64, height: *mut f64) -> i32;
    }

    // milliseconds since start of program
    pub fn now() -> f64 {
        unsafe { emscripten_get_now() }
    }

    pub fn sleep(ms: u32) {
        unsafe {
            emscripten_sleep(ms);
        }
    }

    pub fn get_canvas_size() -> (u32, u32) {
        let mut width = 0.0;
        let mut height = 0.0;
        unsafe {
            emscripten_get_element_css_size("canvas\0".as_ptr(), &mut width, &mut height);
        }
        (width as u32, height as u32)
    }
}
