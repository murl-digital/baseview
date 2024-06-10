use std::{convert::TryInto, ffi::OsStr, mem, os::windows::ffi::OsStrExt, ptr::{null, null_mut}};

use winapi::{shared::{minwindef::{BOOL, LPARAM, LRESULT, WPARAM}, ntdef::PCWSTR, windef::{ HICON, HWND}}, um::winuser::{CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, GetWindowLongPtrW, PostMessageW, RegisterClassW, SetFocus, SetWindowLongPtrW, TranslateMessage, UnregisterClassW, CS_OWNDC, GWLP_USERDATA, VK_CONTROL, VK_DELETE, VK_DOWN, VK_LEFT, VK_MENU, VK_RETURN, VK_RIGHT, VK_SHIFT, VK_UP, WM_CHAR, WM_KEYDOWN, WM_KEYUP, WNDCLASSW, WS_CHILD, WS_EX_NOACTIVATE}};

use super::generate_guid;

pub struct MessageWindow {
    hwnd: HWND,
    main_window_hwnd: HWND,
    window_class: u16,
}

unsafe impl Send for MessageWindow {}
unsafe impl Sync for MessageWindow {}

impl MessageWindow {
    pub fn new(main_window_hwnd: HWND) -> Self {
        let class_name: Vec<u16> = OsStr::new(&("plugin-canvas-message-window-".to_string() + unsafe { &generate_guid() })).encode_wide().collect();
        let window_name: Vec<u16> = OsStr::new("Message window").encode_wide().collect();

        let window_class_attributes = WNDCLASSW {
            style: CS_OWNDC,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: null_mut(),
            hIcon: null_mut(),
            hCursor: null_mut(),
            hbrBackground: null_mut(),
            lpszMenuName: window_name.as_ptr(),
            lpszClassName: class_name.as_ptr()
        };

        let window_class = unsafe { RegisterClassW(&window_class_attributes) };
        if window_class == 0 {
            return panic!("Failed to register window class");
        }

        let hwnd = unsafe { CreateWindowExW(
            WS_EX_NOACTIVATE,
            window_class as _,
            window_name.as_ptr(),
            WS_CHILD,
            0,
            0,
            0,
            0,
            main_window_hwnd,
            null_mut(),
            null_mut(),
            null_mut(),
        ) };

        unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, main_window_hwnd as _) };

        Self {
            hwnd,
            main_window_hwnd,
            window_class,
        }
    }

    pub fn run(&self) {
        unsafe {
            let mut msg = mem::zeroed();

            loop {
                match GetMessageW(&mut msg, self.hwnd, 0, 0) {
                    -1 => {
                        panic!()
                    }

                    0 => {
                        return;
                    }

                    _ => {}
                }

                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }    
        }
    }

    pub fn set_focus(&self, focus: bool) {
        let hwnd = if focus {
            self.hwnd
        } else {
            self.main_window_hwnd
        };

        unsafe { SetFocus(hwnd); }
    }
}

impl Drop for MessageWindow {
    fn drop(&mut self) {
        unsafe {
            DestroyWindow(self.hwnd);
            UnregisterClassW(self.window_class as _, null_mut());
        }
    }
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let main_window_hwnd = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as _};

    match msg {
        WM_CHAR => {
            PostMessageW(main_window_hwnd, WM_KEYDOWN, wparam, lparam).try_into().unwrap()
        },

        WM_KEYDOWN => {
            if let Some(character) = virtual_key_to_char(wparam) {
                PostMessageW(main_window_hwnd, WM_KEYDOWN, character, 0).try_into().unwrap()
            } else {
                0
            }
        }

        WM_KEYUP => {
            if let Some(character) = virtual_key_to_char(wparam) {
                PostMessageW(main_window_hwnd, WM_KEYUP, character, 0).try_into().unwrap()
            } else {
                0
            }
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

fn virtual_key_to_char(key: usize) -> Option<usize> {
    match key as i32 {
        VK_RETURN   => Some(0x000a),
        VK_SHIFT    => Some(0x0010),
        VK_CONTROL  => Some(0x0011),
        VK_MENU     => Some(0x0012),
        VK_DELETE   => Some(0x007f),
        VK_UP       => Some(0xf700),
        VK_DOWN     => Some(0xf701),
        VK_LEFT     => Some(0xf702),
        VK_RIGHT    => Some(0xf703),
        _           => None,
    }
}