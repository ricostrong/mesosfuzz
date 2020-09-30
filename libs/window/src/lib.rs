use std::io;
use std::io::Error;
use std::cell::Cell;
use std::collections::HashSet;
use std::convert::TryInto;
use std::sync::{Arc,Mutex};
use corpus::Corpus;

#[link(name="User32")]
extern "system" {
    fn FindWindowW(lpClassName: *mut u16, lpWindowName: *mut u16) -> usize;
    fn PostMessageW(hWnd: usize, msg: u32, wParam: usize, lParam: usize)
        -> usize;
    fn GetForegroundWindow() -> usize;
    fn SendInput(cInputs: u32, pInputs: *mut Input, cbSize: i32) -> u32;
    fn SetForegroundWindow(hwnd: usize) -> bool;
    fn GetClientRect(hwnd: usize, rect: *mut Rect) -> bool;
    fn GetWindowRect(hwnd: usize, rect: *mut Rect) -> bool;
    fn GetAsyncKeyState(vKey: i32) -> i32;
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct Rect {
    left:   i32,
    top:    i32,
    right:  i32,
    bottom: i32,
}

/// Different types of inpust for the `typ` field on `Input`
#[repr(C)]
#[derive(Clone, Copy)]
pub enum InputType {
    Mouse    = 0,
    Keyboard = 1,
    Hardware = 2,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Input {
    typ:   InputType,
    union: InputUnion,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union InputUnion {
    mouse:    MouseInput,
    keyboard: KeyboardInput,
    hardware: HardwareInput,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeyboardInput {
    vk:          u16,
    scan_code:   u16,
    flags:       u32,
    time:        u32,
    extra_info:  usize,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MouseInput {
    pub dx:         i32,
    pub dy:         i32,
    pub mouse_data: u32,
    pub flags:      u32,
    pub time:       u32,
    pub extra_info: usize,
}



#[repr(C)]
#[derive(Clone, Copy)]
pub struct HardwareInput {
    msg:    u32,
    lparam: u16,
    hparam: u16,
}

/// Convert a Rust UTF-8 `string` into a NUL-terminated UTF-16 vector
fn str_to_utf16(string: &str) -> Vec<u16> {
    let mut ret: Vec<u16> = string.encode_utf16().collect();
    ret.push(0);
    ret
}

/// Different types of messages for `SendMessage()`
#[repr(u32)]
enum MsgType {
    LeftButtonDown = 0x0201,
    LeftButtonUp   = 0x0202,
    KeyDown        = 0x0100,
    KeyUp          = 0x0101,
}

/// Different types of states for the `WPARAM` field on 
#[repr(usize)]
enum WparamMousePress {
    Left     = 0x0001,
    Right    = 0x0002,
    Shift    = 0x0004,
    Control  = 0x0008,
    Middle   = 0x0010,
    Xbutton1 = 0x0020,
    Xbutton2 = 0x0040,
}

#[repr(u8)]
enum KeyCode {
    Back    = 0x08,
    Tab     = 0x09,
    Return  = 0x0d,
    Shift   = 0x10,
    Control = 0x11,
    Alt     = 0x12,
    Left    = 0x25,
    Up      = 0x26,
    Right   = 0x27,
    Down    = 0x28,
}

/// An active handle to a window
pub struct Window {
    /// Handle to the window which we have opened
    pub hwnd: usize,

    /// Keys which seem interesting
    interesting_keys: Vec<u8>,
}

impl Window {
    pub fn new() -> Self{
        //empty window

        Window{
            hwnd: 0,
            interesting_keys: Vec::new(),
        }
    }
    pub fn get_window(windowname: &str) -> Self{
    
        let mut window = Window::attach(windowname);
        /*while window.is_err() {
            print!("could not attach to window {}\n", windowname);
            window = Window::attach(windowname);
            std::thread::sleep_ms(10);
            //return ExitType::Finished();
        }*/
        if window.is_err() {
            return Window::new();
        }
        else{
            return window.unwrap();
        }
    }
    pub fn close_case(&self){

        if unsafe { SetForegroundWindow(self.hwnd) } == false {
            print!("Couldn't set foreground\n");
        }
        
        // ctrl+ w just exits in the case of notepad
        self.ctrl_press(0x57 as u16).expect("could not press this bitch");
    }
    pub fn fuzz_case(&self, filename: &str) {    

        if unsafe { SetForegroundWindow(self.hwnd) } == false {
            print!("Couldn't set foreground\n");
        }
        
        self.ctrl_press(0x4f as u16).expect("could not press this bitch");
    
        let x = String::from(filename).to_uppercase();
        for charr in x.chars() {

            match charr as u8 as u16 {
                // : 
                58 => {
                    self.shift_press(0xBA).expect("could not press this bitch");
                }
                92 => {
                    self.press(0xDC).expect("could not press this bitch");
                }
                0x2e =>{
                    self.press(0xBE).expect("could not press this bitch");
                }
                0x5f =>{
                    self.shift_press(0xBD).expect("could not press this bitch");
                }
                _ => {
                    self.press(charr as u8 as u16).expect("could not press this bitch");
                }
            }
        }
        // enter to accept the file
        self.press(0x0D).expect("could not press this bitch");

        
    }
    pub fn attach(title: &str) -> io::Result<Self> {
        // Convert the title to UTF-16
        let mut title = str_to_utf16(title); 

        // Finds the window with `title`
        let ret = unsafe {
            FindWindowW(std::ptr::null_mut(), title.as_mut_ptr())
        };

        // Generate some interesting keys
        let mut interesting_keys = Vec::new();
        interesting_keys.push(KeyCode::Left  as u8);
        interesting_keys.push(KeyCode::Up    as u8);
        interesting_keys.push(KeyCode::Down  as u8);
        interesting_keys.push(KeyCode::Right as u8);
        interesting_keys.push(KeyCode::Tab   as u8);
        interesting_keys.extend_from_slice(
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789()-+=/*!@#");

        if ret != 0 {
            // Successfully got a handle to the window
            return Ok(Window {
                hwnd: ret,
                interesting_keys,
            });
        } else {
            // FindWindow() failed, return out the corresponding error
            Err(Error::last_os_error())
        }
    }

    pub fn keystream(&self, inputs: &[KeyboardInput]) -> io::Result<()> {
        // Generate an array to pass directly to `SendInput()`
        let mut win_inputs = Vec::new();

        // Create inputs based on each keyboard input
        for &input in inputs.iter() {
            win_inputs.push(Input {
                typ: InputType::Keyboard,
                union: InputUnion {
                    keyboard: input
                }
            });
        }

        let res = unsafe {
            SendInput(
                win_inputs.len().try_into().unwrap(),
                win_inputs.as_mut_ptr(),
                std::mem::size_of::<Input>().try_into().unwrap())
        };

        if (res as usize) != inputs.len() {
            Err(Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub fn mousestream(&self, inputs: &[MouseInput]) -> io::Result<()> {
        // Generate an array to pass directly to `SendInput()`
        let mut win_inputs = Vec::new();

        // Create inputs based on each mouse input
        for &input in inputs.iter() {
            win_inputs.push(Input {
                typ: InputType::Mouse,
                union: InputUnion {
                    mouse: input
                }
            });
        }

        let res = unsafe {
            SendInput(
                win_inputs.len().try_into().unwrap(),
                win_inputs.as_mut_ptr(),
                std::mem::size_of::<Input>().try_into().unwrap())
        };

        if (res as usize) != inputs.len() {
            Err(Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub fn press(&self, key: u16) -> io::Result<()> {
        self.keystream(&[
            KeyboardInput {
                vk: key,
                scan_code: 0,
                flags: 0,
                time: 0,
                extra_info: 0,
            },

            KeyboardInput {
                vk: key,
                scan_code: 0,
                flags: KEYEVENTF_KEYUP,
                time: 0,
                extra_info: 0,
            },
        ])
    }

    pub fn alt_press(&self, key: u16) -> io::Result<()> {
        if key == KeyCode::Tab as u16 || key == b' ' as u16 {
            return Ok(());
        }

        self.keystream(&[
            KeyboardInput {
                vk: KeyCode::Alt as u16,
                scan_code: 0,
                flags: 0,
                time: 0,
                extra_info: 0,
            },

            KeyboardInput {
                vk: key,
                scan_code: 0,
                flags: 0,
                time: 0,
                extra_info: 0,
            },

            KeyboardInput {
                vk: key,
                scan_code: 0,
                flags: KEYEVENTF_KEYUP,
                time: 0,
                extra_info: 0,
            },

            KeyboardInput {
                vk: KeyCode::Alt as u16,
                scan_code: 0,
                flags: KEYEVENTF_KEYUP,
                time: 0,
                extra_info: 0,
            },
        ])
    }

    pub fn shift_press(&self, key: u16) -> io::Result<()> {

        self.keystream(&[
            KeyboardInput {
                vk: KeyCode::Shift as u16,
                scan_code: 0,
                flags: 0,
                time: 0,
                extra_info: 0,
            },

            KeyboardInput {
                vk: key,
                scan_code: 0,
                flags: 0,
                time: 0,
                extra_info: 0,
            },

            KeyboardInput {
                vk: key,
                scan_code: 0,
                flags: KEYEVENTF_KEYUP,
                time: 0,
                extra_info: 0,
            },

            KeyboardInput {
                vk: KeyCode::Shift as u16,
                scan_code: 0,
                flags: KEYEVENTF_KEYUP,
                time: 0,
                extra_info: 0,
            },
        ])
    }

    pub fn ctrl_press(&self, key: u16) -> io::Result<()> {
        if key == 0x1B {
            return Ok(());
        }

        self.keystream(&[
            KeyboardInput {
                vk: KeyCode::Control as u16,
                scan_code: 0,
                flags: 0,
                time: 0,
                extra_info: 0,
            },

            KeyboardInput {
                vk: key,
                scan_code: 0,
                flags: 0,
                time: 0,
                extra_info: 0,
            },

            KeyboardInput {
                vk: key,
                scan_code: 0,
                flags: KEYEVENTF_KEYUP,
                time: 0,
                extra_info: 0,
            },

            KeyboardInput {
                vk: KeyCode::Control as u16,
                scan_code: 0,
                flags: KEYEVENTF_KEYUP,
                time: 0,
                extra_info: 0,
            },
        ])
    }
}

const KEYEVENTF_KEYUP: u32 = 0x0002;

pub fn close_case(window: Window){

    if unsafe { SetForegroundWindow(window.hwnd) } == false {
        print!("Couldn't set foreground\n");
    }
    
    // ctrl+ w just exits in the case of notepad
    window.ctrl_press(0x57 as u16).expect("could not press this bitch");
}




pub fn ifexit() -> bool{
    if unsafe{GetAsyncKeyState(0x7B)} != 0{
        return true;
    }
    return false;   
}