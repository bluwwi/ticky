use crate::audio::AudioHandle;
use crate::packs::Pack;
use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{VK_BACK, VK_RETURN};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
    UnhookWindowsHookEx, HC_ACTION, KBDLLHOOKSTRUCT, LLKHF_INJECTED, LLMHF_INJECTED,
    MSLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN,
    WM_MBUTTONDOWN, WM_MOUSEHWHEEL, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_SYSKEYDOWN, WM_SYSKEYUP,
    WM_XBUTTONDOWN, XBUTTON1,
};

static ENGINE: OnceLock<Arc<Engine>> = OnceLock::new();

#[derive(Default)]
struct SampleSet {
    key_down: Vec<Arc<[u8]>>,
    key_up: Vec<Arc<[u8]>>,
    enter: Vec<Arc<[u8]>>,
    backspace: Vec<Arc<[u8]>>,
    mouse_left: Vec<Arc<[u8]>>,
    mouse_right: Vec<Arc<[u8]>>,
    mouse_middle: Vec<Arc<[u8]>>,
    mouse_back: Vec<Arc<[u8]>>,
    mouse_forward: Vec<Arc<[u8]>>,
    wheel_up: Vec<Arc<[u8]>>,
    wheel_down: Vec<Arc<[u8]>>,
    wheel_left: Vec<Arc<[u8]>>,
    wheel_right: Vec<Arc<[u8]>>,
}

impl SampleSet {
    fn from_pack(pack: &Pack) -> Self {
        let load = |list: &[String]| -> Vec<Arc<[u8]>> {
            list.iter()
                .filter_map(|relative| load_sample(&pack.dir, relative))
                .collect()
        };
        let sounds = &pack.manifest.sounds;

        Self {
            key_down: load(&sounds.key_down),
            key_up: load(&sounds.key_up),
            enter: load(&sounds.enter),
            backspace: load(&sounds.backspace),
            mouse_left: load(&sounds.mouse_left),
            mouse_right: load(&sounds.mouse_right),
            mouse_middle: load(&sounds.mouse_middle),
            mouse_back: load(&sounds.mouse_back),
            mouse_forward: load(&sounds.mouse_forward),
            wheel_up: load(&sounds.mouse_wheel_up),
            wheel_down: load(&sounds.mouse_wheel_down),
            wheel_left: load(&sounds.mouse_wheel_left),
            wheel_right: load(&sounds.mouse_wheel_right),
        }
    }

    fn count(&self) -> usize {
        [
            &self.key_down,
            &self.key_up,
            &self.enter,
            &self.backspace,
            &self.mouse_left,
            &self.mouse_right,
            &self.mouse_middle,
            &self.mouse_back,
            &self.mouse_forward,
            &self.wheel_up,
            &self.wheel_down,
            &self.wheel_left,
            &self.wheel_right,
        ]
        .into_iter()
        .map(Vec::len)
        .sum()
    }
}

fn load_sample(dir: &Path, relative: &str) -> Option<Arc<[u8]>> {
    let path = dir.join(relative);
    match std::fs::read(&path) {
        Ok(bytes) => Some(Arc::from(bytes)),
        Err(err) => {
            eprintln!("ticky: skipping {}: {err}", path.display());
            None
        }
    }
}

pub struct Engine {
    enabled: AtomicBool,
    muted: AtomicBool,
    cursor: AtomicUsize,
    pressed: Mutex<HashSet<u32>>,
    samples: RwLock<SampleSet>,
    audio: RwLock<AudioHandle>,
}

impl Engine {
    pub fn new(audio: AudioHandle) -> Self {
        Self {
            enabled: AtomicBool::new(true),
            muted: AtomicBool::new(false),
            cursor: AtomicUsize::new(0),
            pressed: Mutex::new(HashSet::new()),
            samples: RwLock::new(SampleSet::default()),
            audio: RwLock::new(audio),
        }
    }

    pub fn set_audio(&self, audio: AudioHandle) {
        *self.audio.write().unwrap() = audio;
    }

    pub fn shutdown_audio(&self) {
        self.audio.read().unwrap().shutdown();
    }

    pub fn set_flags(&self, enabled: bool, muted: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
        self.muted.store(muted, Ordering::Relaxed);
    }

    pub fn set_pack(&self, pack: &Pack) -> usize {
        let samples = SampleSet::from_pack(pack);
        let count = samples.count();
        *self.samples.write().unwrap() = samples;
        count
    }

    pub fn preview(&self) -> Result<(), String> {
        let samples = self.samples.read().unwrap();
        let bytes = Self::pick(&self.cursor, &samples.key_down)
            .or_else(|| Self::pick(&self.cursor, &samples.key_up))
            .or_else(|| Self::pick(&self.cursor, &samples.enter))
            .or_else(|| Self::pick(&self.cursor, &samples.mouse_left))
            .ok_or_else(|| "This pack has no playable samples".to_string())?;
        self.audio.read().unwrap().play(bytes, 1.0);
        Ok(())
    }

    fn is_active(&self) -> bool {
        self.enabled.load(Ordering::Relaxed) && !self.muted.load(Ordering::Relaxed)
    }

    fn pick(cursor: &AtomicUsize, list: &[Arc<[u8]>]) -> Option<Arc<[u8]>> {
        if list.is_empty() {
            None
        } else {
            let index = cursor.fetch_add(1, Ordering::Relaxed) % list.len();
            Some(list[index].clone())
        }
    }

    fn emit(&self, list: &[Arc<[u8]>]) {
        if let Some(bytes) = Self::pick(&self.cursor, list) {
            self.audio.read().unwrap().play(bytes, 1.0);
        }
    }

    fn on_key(&self, vk: u32, down: bool) {
        let mut pressed = self.pressed.lock().unwrap();
        if down {
            if !pressed.insert(vk) {
                return;
            }
        } else {
            pressed.remove(&vk);
            return;
        }
        drop(pressed);

        if !self.is_active() {
            return;
        }

        let samples = self.samples.read().unwrap();
        let list = if vk == VK_RETURN.0 as u32 && !samples.enter.is_empty() {
            &samples.enter
        } else if vk == VK_BACK.0 as u32 && !samples.backspace.is_empty() {
            &samples.backspace
        } else {
            &samples.key_down
        };

        self.emit(list);
    }

    fn on_mouse(&self, message: u32, mouse_data: u32) {
        if !self.is_active() {
            return;
        }

        let samples = self.samples.read().unwrap();
        let list = match message {
            WM_LBUTTONDOWN => &samples.mouse_left,
            WM_RBUTTONDOWN => &samples.mouse_right,
            WM_MBUTTONDOWN => &samples.mouse_middle,
            WM_XBUTTONDOWN => {
                if (mouse_data >> 16) as u16 == XBUTTON1 {
                    &samples.mouse_back
                } else {
                    &samples.mouse_forward
                }
            }
            WM_MOUSEWHEEL => {
                if (mouse_data >> 16) as u16 as i16 > 0 {
                    &samples.wheel_up
                } else {
                    &samples.wheel_down
                }
            }
            WM_MOUSEHWHEEL => {
                if (mouse_data >> 16) as u16 as i16 > 0 {
                    &samples.wheel_right
                } else {
                    &samples.wheel_left
                }
            }
            _ => return,
        };

        self.emit(list);
    }
}

pub fn start(engine: Arc<Engine>) {
    if ENGINE.set(engine).is_err() {
        return;
    }

    let _ = std::thread::Builder::new()
        .name("ticky-hooks".into())
        .spawn(hook_thread);
}

fn hook_thread() {
    unsafe {
        let keyboard = match SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), None, 0) {
            Ok(hook) => Some(hook),
            Err(err) => {
                eprintln!("ticky: keyboard hook unavailable: {err}");
                None
            }
        };
        let mouse = match SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), None, 0) {
            Ok(hook) => Some(hook),
            Err(err) => {
                eprintln!("ticky: mouse hook unavailable: {err}");
                None
            }
        };

        if keyboard.is_none() && mouse.is_none() {
            return;
        }
        eprintln!("ticky: input hooks installed");

        let mut message = MSG::default();
        loop {
            let result = GetMessageW(&mut message, None, 0, 0);
            if result.0 <= 0 {
                break;
            }
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }

        if let Some(hook) = keyboard {
            let _ = UnhookWindowsHookEx(hook);
        }
        if let Some(hook) = mouse {
            let _ = UnhookWindowsHookEx(hook);
        }
    }
}

unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        let info = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        if info.flags.0 & LLKHF_INJECTED.0 == 0 {
            let message = wparam.0 as u32;
            let down = message == WM_KEYDOWN || message == WM_SYSKEYDOWN;
            let up = message == WM_KEYUP || message == WM_SYSKEYUP;

            if down || up {
                if let Some(engine) = ENGINE.get() {
                    engine.on_key(info.vkCode, down);
                }
            }
        }
    }

    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        let info = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
        if info.flags & LLMHF_INJECTED == 0 {
            if let Some(engine) = ENGINE.get() {
                engine.on_mouse(wparam.0 as u32, info.mouseData);
            }
        }
    }

    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}
