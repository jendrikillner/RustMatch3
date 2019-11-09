use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use winapi::shared::minwindef::{HINSTANCE, LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::ntdef::LPCWSTR;
use winapi::shared::windef::{HBRUSH, HICON, HMENU, HWND};
use winapi::um::winuser::*;

fn as_fractional_secs(dur: &std::time::Duration) -> f32 {
    (dur.as_secs() as f64 + f64::from(dur.subsec_nanos()) / 1_000_000_000.0) as f32
}

enum WindowMessages {
    WindowClosed,
}

struct Window {
    hwnd: HWND,
}

static mut IS_WINDOW_CLOSED: bool = false;

unsafe extern "system" fn window_proc(
    h_wnd: HWND,
    msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if msg == WM_DESTROY {
        IS_WINDOW_CLOSED = true;

        PostQuitMessage(0);
    }

    DefWindowProcW(h_wnd, msg, w_param, l_param)
}

fn create_window() -> Result<Window, ()> {
    unsafe {
        let mut window_class_name: Vec<u16> =
            OsStr::new("Match3WindowClass").encode_wide().collect();

        window_class_name.push(0);

        let window_class = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: 0 as HINSTANCE,
            hIcon: 0 as HICON,
            hCursor: 0 as HICON,
            hbrBackground: 16 as HBRUSH,
            lpszMenuName: 0 as LPCWSTR,
            lpszClassName: window_class_name.as_ptr(),
        };

        let error_code = RegisterClassW(&window_class);

        assert!(error_code != 0, "failed to register the window class");

        let h_wnd_window = CreateWindowExW(
            0,
            window_class_name.as_ptr(),
            0 as LPCWSTR,
            WS_OVERLAPPED | WS_MINIMIZEBOX | WS_SYSMENU,
            0,
            0,
            400,
            400,
            0 as HWND,
            0 as HMENU,
            0 as HINSTANCE,
            std::ptr::null_mut(),
        );

        assert!(h_wnd_window != (0 as HWND), "failed to open the window");

        ShowWindow(h_wnd_window, SW_SHOW);

        Ok(Window { hwnd: h_wnd_window })
    }
}

fn process_window_messages(window: &Window) -> Option<WindowMessages> {
    unsafe {
        let mut msg: MSG = std::mem::zeroed();

        // process messages
        while PeekMessageA(&mut msg, window.hwnd, 0, 0, PM_REMOVE) > 0 {
            TranslateMessage(&msg);
            DispatchMessageA(&msg);

            if IS_WINDOW_CLOSED {
                return Some(WindowMessages::WindowClosed);
            }
        }

        None
    }
}

fn main() {
    let mut should_game_close = false;

    // afterwards open a window we can render into
    let _main_window: Window = create_window().unwrap();

    let dt: f32 = 1.0 / 60.0;
    let mut accumulator: f32 = dt;

    let mut current_time = std::time::Instant::now();
    let mut draw_frame_number: u64 = 0;
    let mut update_frame_number: u64 = 0;

    while !should_game_close {
        let new_time = std::time::Instant::now();

        while let Some(x) = process_window_messages(&_main_window) {
            match x {
                WindowMessages::WindowClosed => {
                    should_game_close = true;
                }
            }
        }

        // calculate how much time has passed
        let frame_time = f32::min(
            as_fractional_secs(&new_time.duration_since(current_time)),
            0.25,
        );

        accumulator += frame_time;

        println!("frame time {}", frame_time);

        current_time = new_time;

        while accumulator >= dt {
            println!(
                "update {} accumulator {} dt {} ",
                update_frame_number, accumulator, dt
            );

            // update the game for a fixed number of steps
            accumulator -= dt;
            update_frame_number += 1;
        }

        // draw the game
        let subframe_blend = accumulator / dt;

        // draw
        println!(
            "draw {} subframe_blend {}",
            draw_frame_number, subframe_blend
        );

        draw_frame_number += 1;
    }
}
