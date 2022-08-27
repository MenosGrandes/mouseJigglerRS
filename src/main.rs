
#![cfg(windows)]
// Let's put this so that it won't open the console
#![windows_subsystem = "windows"]

/// Based on https://github.com/pachi/rust_winapi_examples and
/// https://gist.github.com/littletsu/d1c1b512d6843071144b7b89109a8de2
use std::error::Error;
use std::ptr::null_mut;
use winapi::shared::minwindef::*;
use winapi::shared::ntdef::*;
use winapi::shared::windef::*;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::*;
mod jiggler;

// Global Model to keep state
struct Model {
    h_btn_prj_out: HWND,
}

static mut MODEL: Model = Model {
    h_btn_prj_out: 0 as HWND,
};

// Control IDs
static mut OPOSITE_MOVE: i8 = -1;
const CHECKBOX_RUN: WORD = 111;
const TIMER_ID : usize = 112;

// Get a win32 lpstr from a &str, converting u8 to u16 and appending '\0'
fn to_wstring(value: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;

    std::ffi::OsStr::new(value)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

// Get a String from a string as wide pointer (PWSTR)
pub unsafe fn pwstr_to_string(ptr: PWSTR) -> String {
    use std::slice::from_raw_parts;
    let len = (0_usize..)
        .find(|&n| *ptr.offset(n as isize) == 0)
        .expect("Couldn't find null terminator");
    let array: &[u16] = from_raw_parts(ptr, len);
    String::from_utf16_lossy(array)
}

// Window procedure function to handle events
pub unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CLOSE => {
            DestroyWindow(hwnd);
        }
        WM_DESTROY => {
            PostQuitMessage(0);
        }
        WM_COMMAND => {
            let wm_command_id = LOWORD(wparam as DWORD);
            let wm_event_type = HIWORD(wparam as DWORD);

            match wm_command_id {
                CHECKBOX_RUN => match wm_event_type {
                    BN_CLICKED => {
                        if SendDlgItemMessageW(
                            hwnd,
                            (CHECKBOX_RUN as u32).try_into().unwrap(),
                            BM_GETCHECK,
                            0,
                            0,
                        ) != 0
                        {
                            let call_back : TIMERPROC = Some(timer_callback);
                            SetTimer(hwnd, TIMER_ID, 1000*2, call_back);

                        } else {
                            KillTimer(hwnd,TIMER_ID);
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
    }
    0
}

// Declare class and instantiate window
fn create_main_window(name: &str, title: &str) -> Result<HWND, Box<dyn Error>> {
    let name = to_wstring(name);
    let title = to_wstring(title);

    unsafe {
        // Get handle to the file used to create the calling process
        let hinstance = GetModuleHandleW(null_mut());

        // Create and register window class
        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance, // Handle to the instance that contains the window procedure for the class
            hIcon: LoadIconW(null_mut(), IDI_APPLICATION),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: COLOR_WINDOW as HBRUSH,
            lpszMenuName: null_mut(),
            lpszClassName: name.as_ptr(),
            hIconSm: LoadIconW(null_mut(), IDI_APPLICATION),
        };

        // Register window class
        if RegisterClassExW(&wnd_class) == 0 {
            MessageBoxW(
                null_mut(),
                to_wstring("Window Registration Failed!").as_ptr(),
                to_wstring("Error").as_ptr(),
                MB_ICONEXCLAMATION | MB_OK,
            );
            return Err("Window Registration Failed".into());
        };

        // Create a window based on registered class
        let handle = CreateWindowExW(
            0,                                // dwExStyle
            name.as_ptr(),                    // lpClassName
            title.as_ptr(),                   // lpWindowName
            WS_OVERLAPPEDWINDOW | WS_VISIBLE, // dwStyle
            CW_USEDEFAULT,                    // Int x
            CW_USEDEFAULT,                    // Int y
            630,                              // Int nWidth
            270,                              // Int nHeight
            null_mut(),                       // hWndParent
            null_mut(),                       // hMenu
            hinstance,                        // hInstance
            null_mut(),                       // lpParam
        );

        if handle.is_null() {
            MessageBoxW(
                null_mut(),
                to_wstring("Window Creation Failed!").as_ptr(),
                to_wstring("Error!").as_ptr(),
                MB_ICONEXCLAMATION | MB_OK,
            );
            return Err("Window Creation Failed!".into());
        }

        // Custom GUI
        create_gui(handle);

        ShowWindow(handle, SW_SHOW);
        UpdateWindow(handle);

        Ok(handle)
    }
}

// Build GUI elements inside main window
unsafe fn create_gui(hparent: HWND) {
    let hinstance = GetWindowLongW(hparent, GWL_HINSTANCE) as HINSTANCE;

    MODEL.h_btn_prj_out = CreateWindowExW(
        0,
        to_wstring("button").as_ptr(),
        to_wstring("Jiggling?").as_ptr(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_AUTOCHECKBOX | BS_TEXT,
        10,  // x
        10,  // y
        300, // w
        30,  // h
        hparent,
        CHECKBOX_RUN as HMENU,
        hinstance,
        null_mut(),
    );
}

unsafe extern "system" fn timer_callback(hwnd : *mut HWND__ , b :u32 , c: usize, d: u32 ) 
{

    
        jiggler::mouse_jiggler::jiggle(4*OPOSITE_MOVE as i32); 
        OPOSITE_MOVE = OPOSITE_MOVE * -1;

}

// Message handling loop
fn run_message_loop(hwnd: HWND) -> WPARAM {
    unsafe {
        let mut msg: MSG = std::mem::MaybeUninit::zeroed().assume_init();
        loop {
            // Get message from message queue
            if GetMessageW(&mut msg, hwnd, 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            } else {
                // Return on error (<0) or exit (=0) cases
                return msg.wParam;
            }
        }
    }
}

fn main() {
    let hwnd = create_main_window("MouseJiggle", "MouseJiggleRS")
        .expect("Window creation failed!");
        
    run_message_loop(hwnd);
}
