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
    WindowCreated,
    WindowClosed,
}

struct Window {
    message_receiver: std::sync::mpsc::Receiver<WindowMessages>,
}

struct WindowThreadState {
    message_sender: std::sync::mpsc::Sender<WindowMessages>,
    is_window_closed : bool
}

unsafe extern "system" fn window_proc(
    h_wnd: HWND,    
    msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {

    if msg == WM_CREATE {
        // retrieve the message struct that contains the creation parameters
        let pCreate = l_param as * mut winapi::um::winuser::CREATESTRUCTW;
    
        // retrieve the rust window state
        let windowState = pCreate.as_ref().unwrap().lpCreateParams as * mut WindowThreadState;

        // the state we can store inside the user data parameter of the window
        SetWindowLongPtrW(h_wnd, GWLP_USERDATA, windowState as isize );

        windowState.as_mut().unwrap().message_sender.send(WindowMessages::WindowCreated).unwrap();
    }

    let windowState = GetWindowLongPtrW(h_wnd, GWLP_USERDATA) as * mut WindowThreadState;

    if msg == WM_DESTROY {
        windowState.as_mut().unwrap().message_sender.send(WindowMessages::WindowClosed).unwrap();
        windowState.as_mut().unwrap().is_window_closed = true;

        PostQuitMessage(0);
    }

    DefWindowProcW(h_wnd, msg, w_param, l_param)
}

fn create_window() -> Result<Window, ()> {
    let (channel_sender, channel_receiver) = std::sync::mpsc::channel();

    std::thread::spawn(move || {

        let mut window_state = WindowThreadState { 
            message_sender : channel_sender, 
            is_window_closed : false 
            };

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
                & mut window_state as * mut WindowThreadState as * mut winapi::ctypes::c_void, // pass a mutable pointer to the window
            );

            assert!(h_wnd_window != (0 as HWND), "failed to open the window");

            ShowWindow(h_wnd_window, SW_SHOW);

            // channel_sender.send(WindowMessages::WindowCreated).unwrap();

            let mut msg: MSG = std::mem::zeroed();

            // process messages
            while !window_state.is_window_closed {
                if PeekMessageA(&mut msg, h_wnd_window, 0, 0, PM_REMOVE) > 0 {
                    TranslateMessage(&msg);
                    DispatchMessageA(&msg);
                }
            }
        }
    });

    // wait for window created before returning
    if let WindowMessages::WindowCreated = channel_receiver.recv().unwrap() {
        return Ok(Window {
            message_receiver: channel_receiver,
        });
    }

    Err(())
}

fn process_window_messages(window: &Window) -> Option<WindowMessages> {
    if let Ok(x) = window.message_receiver.try_recv() {
        return Some(x);
    }

    None
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
                WindowMessages::WindowCreated => {
                    panic!();
                } // this should never happen
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
