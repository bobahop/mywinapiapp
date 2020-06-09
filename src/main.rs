#![windows_subsystem = "windows"]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(non_snake_case)]
extern crate winapi;
use std::convert::AsMut;
use std::convert::AsRef;
use std::ffi::OsStr;
use std::io::Error;
use std::iter::once;
use std::mem;
use std::mem::{size_of, zeroed};
use std::os::raw::{c_long, c_void};
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;

use self::winapi::shared::basetsd::LONG_PTR;
use self::winapi::shared::minwindef::{
    BOOL, HIWORD, HMODULE, LOWORD, LPARAM, LRESULT, UINT, WPARAM,
};
use self::winapi::shared::windef::{HBRUSH, HCURSOR, HWND};
use self::winapi::um::errhandlingapi::GetLastError;
use self::winapi::um::libloaderapi::GetModuleHandleW;
use self::winapi::um::wingdi::TextOutA;

use self::winapi::um::winuser::{
    BeginPaint, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, EndPaint,
    FillRect, GetCursor, GetMessageW, GetWindowLongPtrW, LoadCursorW, MessageBoxW, PostQuitMessage,
    RegisterClassW, SetCursor, SetWindowLongPtrW, ShowCursor, TrackMouseEvent, TranslateMessage,
};
use self::winapi::um::winuser::{
    COLOR_WINDOW, CREATESTRUCTW, CS_HREDRAW, CS_OWNDC, CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA,
    HTCLIENT, IDC_ARROW, IDOK, MB_OKCANCEL, MSG, TME_LEAVE, TRACKMOUSEEVENT, WM_CLOSE, WM_CREATE,
    WM_DESTROY, WM_MOUSELEAVE, WM_MOUSEMOVE, WM_PAINT, WM_SETCURSOR, WM_SIZE, WNDCLASSW,
    WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

// ----------------------------------------------------

// We have to encode text to wide format for Windows
//#[cfg(windows)]
fn win32_string(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}

// Window struct
//#[cfg(windows)]
struct Window {
    handle: HWND,
}

struct BobWindow {
    inWindow: isize,
}

unsafe extern "system" fn MyWindowProcW(
    hWnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    match Msg {
        WM_CREATE => {
            let bw = (*(lParam as *mut CREATESTRUCTW)).lpCreateParams as *mut BobWindow;
            let success = SetWindowLongPtrW(hWnd, GWLP_USERDATA, bw as isize);
            0
        }
        WM_CLOSE => {
            if MessageBoxW(
                hWnd,
                win32_string("Really quit?").as_ptr(),
                win32_string("Are you serious?").as_ptr(),
                MB_OKCANCEL,
            ) == IDOK
            {
                DestroyWindow(hWnd);
            }
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        WM_PAINT => {
            let mut ps = zeroed();
            let hdc = BeginPaint(hWnd, &mut ps);
            FillRect(hdc, &ps.rcPaint, (COLOR_WINDOW + 1) as HBRUSH);
            TextOutA(
                hdc,
                5,
                5,
                "pigby".as_ptr() as *const i8,
                "pigby".len() as i32,
            );
            EndPaint(hWnd, &mut ps);
            0
        }
        // WM_SIZE => {
        //     let width = LOWORD(lParam as u32);
        //     let height = HIWORD(lParam as u32);
        //     0
        // }
        WM_MOUSELEAVE => {
            let bw = GetWindowLongPtrW(hWnd, GWLP_USERDATA) as *mut BobWindow;
            (*bw).inWindow = 0;
            0
        }
        WM_MOUSEMOVE => {
            let bw = GetWindowLongPtrW(hWnd, GWLP_USERDATA) as *mut BobWindow;
            if (*bw).inWindow == 1 {
                return 0;
            }
            (*bw).inWindow = 1;
            let mut tme: TRACKMOUSEEVENT =
                *(mem::MaybeUninit::<TRACKMOUSEEVENT>::uninit().as_ptr() as *mut TRACKMOUSEEVENT);
            tme.hwndTrack = hWnd;
            tme.dwFlags = TME_LEAVE;
            tme.dwHoverTime = 1;
            tme.cbSize = size_of::<TRACKMOUSEEVENT>() as u32;
            TrackMouseEvent(&mut tme as *mut _ as *mut TRACKMOUSEEVENT);
            SetCursor(LoadCursorW(null_mut(), IDC_ARROW));
            0
        }
        _ => DefWindowProcW(hWnd, Msg, wParam, lParam),
    }
}

fn create_window(hinstance: HMODULE, name: &str, title: &str) -> Result<Window, Error> {
    let name = win32_string(name);
    let title = win32_string(title);
    let bobbie = Box::new(BobWindow { inWindow: 0 });

    unsafe {
        // Create handle instance that will call GetModuleHandleW, which grabs the instance handle of WNDCLASSW (check third parameter)
        //let hinstance = GetModuleHandleW(null_mut());

        // Create "class" for window, using WNDCLASSW struct (different from Window our struct)
        let wnd_class = WNDCLASSW {
            style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW, // Style
            lpfnWndProc: Some(MyWindowProcW), // The callbackfunction for any window event that can occur in our window!!! Here you could react to events like WM_SIZE or WM_QUIT.
            hInstance: hinstance, // The instance handle for our application which we can retrieve by calling GetModuleHandleW.
            lpszClassName: name.as_ptr(), // Our class name which needs to be a UTF-16 string (defined earlier before unsafe). as_ptr() (Rust's own function) returns a raw pointer to the slice's buffer
            cbClsExtra: 0,
            cbWndExtra: 0,
            hIcon: null_mut(),
            hCursor: null_mut(),
            hbrBackground: null_mut(),
            lpszMenuName: null_mut(),
        };

        // We have to register this class for Windows to use
        RegisterClassW(&wnd_class);

        // More info: https://msdn.microsoft.com/en-us/library/windows/desktop/ms632680(v=vs.85).aspx
        // Create a window based on registered class
        let handle = CreateWindowExW(
            0,                                // dwExStyle
            name.as_ptr(), // lpClassName, name of the class that we want to use for this window, which will be the same that we have registered before.
            title.as_ptr(), // lpWindowName
            WS_OVERLAPPEDWINDOW | WS_VISIBLE, // dwStyle
            CW_USEDEFAULT, // Int x
            CW_USEDEFAULT, // Int y
            CW_USEDEFAULT, // Int nWidth
            CW_USEDEFAULT, // Int nHeight
            null_mut(),    // hWndParent
            null_mut(),    // hMenu
            hinstance,     // hInstance
            //&mut bobbie as *BobWindow as *mut c_void, // lpParam
            Box::into_raw(bobbie) as *mut BobWindow as *mut c_void,
        );

        if handle.is_null() {
            Err(Error::last_os_error())
        } else {
            Ok(Window { handle })
        }
    }
}

// Create message handling function with which to link to hook window to Windows messaging system
// More info: https://msdn.microsoft.com/en-us/library/windows/desktop/ms644927(v=vs.85).aspx
fn handle_message(window: &mut Window) -> bool {
    unsafe {
        //let mut message: MSG = mem::uninitialized();
        let message = mem::MaybeUninit::<MSG>::uninit();
        // Get message from message queue with GetMessageW
        if GetMessageW(message.as_ptr() as *mut MSG, window.handle, 0, 0) > 0 {
            TranslateMessage(message.as_ptr() as *const MSG); // Translate message into something meaningful with TranslateMessage
            DispatchMessageW(message.as_ptr() as *const MSG); // Dispatch message with DispatchMessageW

            true
        } else {
            false
        }
    }
}

fn main() {
    //print_message("Hello, world!").unwrap();
    show_window();
}

fn show_window() {
    //Create handle instance that will call GetModuleHandleW, which grabs the instance handle of WNDCLASSW (check third parameter)
    let hinstance = unsafe { GetModuleHandleW(null_mut()) };

    let mut window = create_window(hinstance, "BobsWindow", "Bob Hoeppner's Window").unwrap();
    while handle_message(&mut window) {}
}
