use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use winapi::shared::minwindef::{HINSTANCE, LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::ntdef::LPCWSTR;
use winapi::shared::windef::{HBRUSH, HICON, HMENU, HWND};
use winapi::um::winuser::*;

pub struct WindowCreatedData {
    pub hwnd: HWND,
}

unsafe impl std::marker::Send for WindowCreatedData {}

pub enum WindowMessages {
    WindowCreated(WindowCreatedData),
    WindowClosed,
}

pub struct Window {
    message_receiver: std::sync::mpsc::Receiver<WindowMessages>,
    pub hwnd: HWND,
}

pub struct WindowThreadState {
    pub message_sender: std::sync::mpsc::Sender<WindowMessages>,
}

unsafe extern "system" fn window_proc(
    h_wnd: HWND,
    msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if msg == WM_CREATE {
        // retrieve the message struct that contains the creation parameters
        let create_struct = l_param as *mut winapi::um::winuser::CREATESTRUCTW;

        // retrieve the rust window state
        let window_state_ptr =
            create_struct.as_ref().unwrap().lpCreateParams as *mut WindowThreadState;
        let window_state: &mut WindowThreadState = window_state_ptr.as_mut().unwrap();

        // the state we can store inside the user data parameter of the window
        SetWindowLongPtrW(h_wnd, GWLP_USERDATA, window_state_ptr as isize);

        window_state
            .message_sender
            .send(WindowMessages::WindowCreated(WindowCreatedData {
                hwnd: h_wnd,
            }))
            .unwrap();
    }

    if msg == WM_DESTROY {
        let window_state_ptr = GetWindowLongPtrW(h_wnd, GWLP_USERDATA) as *mut WindowThreadState;
        let window_state: &mut WindowThreadState = window_state_ptr.as_mut().unwrap();

        window_state
            .message_sender
            .send(WindowMessages::WindowClosed)
            .unwrap();

        PostQuitMessage(0);
    }

    DefWindowProcW(h_wnd, msg, w_param, l_param)
}

pub fn create_window() -> Result<Window, ()> {
    let (channel_sender, channel_receiver) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let mut window_state = WindowThreadState {
            message_sender: channel_sender,
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
                &mut window_state as *mut WindowThreadState as *mut winapi::ctypes::c_void, // pass a mutable pointer to the window
            );

            assert!(h_wnd_window != (0 as HWND), "failed to open the window");

            ShowWindow(h_wnd_window, SW_SHOW);

            let mut msg: MSG = std::mem::zeroed();

            // process messages
            loop {
                if PeekMessageA(&mut msg, h_wnd_window, 0, 0, PM_REMOVE) > 0 {
                    TranslateMessage(&msg);
                    DispatchMessageA(&msg);

                    // once the window has been closed we can exit the message loop
                    if msg.message == WM_QUIT {
                        break;
                    }
                }
            }
        }
    });

    // wait for window created before returning
    if let WindowMessages::WindowCreated(x) = channel_receiver.recv().unwrap() {
        return Ok(Window {
            message_receiver: channel_receiver,
            hwnd: x.hwnd,
        });
    }

    Err(())
}

pub fn process_window_messages(window: &Window) -> Option<WindowMessages> {
    if let Ok(x) = window.message_receiver.try_recv() {
        return Some(x);
    }

    None
}
