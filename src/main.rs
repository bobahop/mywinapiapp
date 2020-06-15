#![windows_subsystem = "windows"]
//#![allow(unused_imports)]
//#![allow(unused_variables)]
//#![allow(dead_code)]
#![allow(non_snake_case)]
extern crate winapi;
use std::ffi::OsStr;
use std::io::Error;
use std::iter::once;
use std::mem;
use std::mem::{size_of, zeroed};
use std::os::raw::c_void;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;

//use self::winapi::shared::basetsd::LONG_PTR;
use self::winapi::shared::minwindef::{HMODULE, LOWORD, LPARAM, LRESULT, TRUE, UINT, WPARAM};
use self::winapi::shared::windef::{HBRUSH, HMENU, HWND};
//use self::winapi::um::errhandlingapi::GetLastError;
use self::winapi::shared::windef::RECT;
use self::winapi::um::libloaderapi::GetModuleHandleW;
use self::winapi::um::wingdi::TextOutA;
use self::winapi::um::winuser::{
    BeginPaint, CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, EndPaint,
    FillRect, GetMessageW, GetWindowLongPtrW, InvalidateRect, LoadCursorW, MessageBoxW,
    PostQuitMessage, RegisterClassW, SetCursor, SetWindowLongPtrW, TrackMouseEvent,
    TranslateMessage, UpdateWindow,
};
use self::winapi::um::winuser::{
    BS_DEFPUSHBUTTON, COLOR_WINDOW, CREATESTRUCTW, CS_HREDRAW, CS_OWNDC, CS_VREDRAW, CW_USEDEFAULT,
    GWLP_USERDATA, IDC_ARROW, IDOK, MB_OK, MB_OKCANCEL, MSG, TME_LEAVE, TRACKMOUSEEVENT, WM_CLOSE,
    WM_COMMAND, WM_CREATE, WM_DESTROY, WM_MOUSELEAVE, WM_MOUSEMOVE, WM_PAINT, WNDCLASSW, WS_CHILD,
    WS_OVERLAPPEDWINDOW, WS_TABSTOP, WS_VISIBLE,
};

// ----------------------------------------------------

fn wstr(value: &str) -> Vec<u16> {
    //converts str to a utf-16 Vector and appends null terminator
    OsStr::new(value).encode_wide().chain(once(0)).collect()
}

struct Window {
    handle: HWND,
}

struct BobWindow {
    inWindow: isize,
}

enum BtnId {
    Btn1 = 1,
    Btn2 = 2,
}
impl BtnId {
    fn from_u32(value: u32) -> BtnId {
        match value {
            1 => BtnId::Btn1,
            2 => BtnId::Btn2,
            _ => panic!("Unknown value {}", value),
        }
    }
}

fn btn1_click(hWnd: HWND) {
    unsafe {
        MessageBoxW(
            hWnd,
            wstr("You touched me!").as_ptr(),
            wstr("Clicked button 1!").as_ptr(),
            MB_OK,
        );
    }
}

fn btn2_click(hWnd: HWND) {
    unsafe {
        UpdateWindow(hWnd);
    }
}

//fn open_file_dialog(hWnd: HWND) {}

fn is_mouse_in(hWnd: HWND) -> bool {
    unsafe {
        let bw = GetWindowLongPtrW(hWnd, GWLP_USERDATA) as *mut BobWindow;
        match (*bw).inWindow {
            0 => false,
            1 => true,
            _ => false,
        }
    }
}

fn get_rect() -> RECT {
    RECT {
        left: 0,
        top: 0,
        right: 125,
        bottom: 25,
    }
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
            SetWindowLongPtrW(hWnd, GWLP_USERDATA, bw as isize);
            create_button(wstr("button1"), BtnId::Btn1, 50, 100, 100, 25, hWnd);
            create_button(wstr("button2"), BtnId::Btn2, 250, 100, 100, 25, hWnd);
            0
        }
        WM_CLOSE => {
            if MessageBoxW(
                hWnd,
                wstr("Really quit?").as_ptr(),
                wstr("Are you serious?").as_ptr(),
                MB_OKCANCEL,
            ) == IDOK
            {
                DestroyWindow(hWnd);
            }
            0
        }
        WM_COMMAND => {
            let id = BtnId::from_u32(LOWORD(wParam as u32) as u32);
            match id {
                BtnId::Btn1 => {
                    btn1_click(hWnd);
                }
                BtnId::Btn2 => {
                    btn2_click(hWnd);
                }
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
            let mouse_status = format!("{}{}", "mouse is in: ", is_mouse_in(hWnd));
            TextOutA(
                hdc,
                5,
                5,
                mouse_status.as_ptr() as *const i8,
                mouse_status.len() as i32,
            );
            EndPaint(hWnd, &mut ps);
            0
        }
        WM_MOUSELEAVE => {
            let bw = GetWindowLongPtrW(hWnd, GWLP_USERDATA) as *mut BobWindow;
            (*bw).inWindow = 0;
            let myupdate = get_rect();
            InvalidateRect(hWnd, &myupdate, TRUE);
            UpdateWindow(hWnd);
            0
        }
        WM_MOUSEMOVE => {
            //only want to set cursor and mouse event once
            if is_mouse_in(hWnd) {
                return 0;
            }
            let bw = GetWindowLongPtrW(hWnd, GWLP_USERDATA) as *mut BobWindow;
            (*bw).inWindow = 1;
            //more info: https://www.codeproject.com/Questions/279139/Mouse-leave-message-is-not-received-when-it-leaves
            //WM_MOUSELEAVE would not fire without this
            let mut tme: TRACKMOUSEEVENT =
                *(mem::MaybeUninit::<TRACKMOUSEEVENT>::uninit().as_ptr() as *mut TRACKMOUSEEVENT);
            tme.hwndTrack = hWnd;
            tme.dwFlags = TME_LEAVE;
            tme.dwHoverTime = 1;
            tme.cbSize = size_of::<TRACKMOUSEEVENT>() as u32;
            TrackMouseEvent(&mut tme as *mut _ as *mut TRACKMOUSEEVENT);
            //Had to do this or cursor was eternal spinny on startup.
            //If it moved out of client area to top of window it turned into arrow
            //and stayed arrow when returned to the client area,
            //But if moved left, right, or down out of client area and returned to client area
            //it turned into and stayed a resize cursor.
            SetCursor(LoadCursorW(null_mut(), IDC_ARROW));
            let myupdate = get_rect();
            InvalidateRect(hWnd, &myupdate, TRUE);
            UpdateWindow(hWnd);
            0
        }
        _ => DefWindowProcW(hWnd, Msg, wParam, lParam),
    }
}

fn create_button(btn_name: Vec<u16>, id: BtnId, x: i32, y: i32, w: i32, h: i32, parent: HWND) {
    unsafe {
        CreateWindowExW(
            0,
            wstr("button").as_ptr(),
            btn_name.as_ptr(),
            WS_TABSTOP | WS_VISIBLE | WS_CHILD | BS_DEFPUSHBUTTON,
            x,
            y,
            w,
            h,
            parent,
            id as u32 as HMENU,
            GetModuleHandleW(null_mut()),
            null_mut(),
        );
    }
}

fn create_window(hinstance: HMODULE, name: &str, title: &str) -> Result<Window, Error> {
    let name = wstr(name);
    let title = wstr(title);
    //need to box the struct onto the heap
    //or inWindow will always have the same value as LONG_PTR returned by GetWindowLongPtrW
    let bobbie = Box::new(BobWindow { inWindow: 0 });

    unsafe {
        //More info: https://docs.microsoft.com/en-us/windows/win32/learnwin32/creating-a-window
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
            //now that it's on the heap, unbox the struct into a pointer
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
        let message = mem::MaybeUninit::<MSG>::uninit();
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
    show_window();
}

fn show_window() {
    //Create handle instance that will call GetModuleHandleW, which grabs the instance handle of WNDCLASSW (check third parameter)
    let hinstance = unsafe { GetModuleHandleW(null_mut()) };

    let mut window = create_window(hinstance, "BobsWindow", "Bob Hoeppner's Window").unwrap();
    while handle_message(&mut window) {}
}
