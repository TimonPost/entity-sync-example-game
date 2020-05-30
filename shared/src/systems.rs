use std::sync::{Mutex, Arc, MutexGuard, LockResult};
use sdl2::render::WindowCanvas;
use sdl2::EventPump;
use sdl2::ttf::Sdl2TtfContext;

pub struct WindowResource {
    window: Arc<Mutex<WindowCanvas>>,
    event_pump: Arc<Mutex<EventPump>>,
    tff_context: Arc<Mutex<Sdl2TtfContext>>
}

unsafe impl Send for WindowResource{}
unsafe impl Sync for WindowResource{}

impl WindowResource {
    pub fn new(window: WindowCanvas, event_pump: EventPump) -> WindowResource {
        WindowResource {
            window: Arc::new(Mutex::new(window)),
            event_pump: Arc::new(Mutex::new(event_pump)),
            tff_context: Arc::new(Mutex::new(sdl2::ttf::init().map_err(|e| e.to_string()).unwrap()))
        }
    }

    pub fn window_lock(&self) -> LockResult<MutexGuard<WindowCanvas>> {
        self.window.lock()
    }

    pub fn event_pump(&self) -> LockResult<MutexGuard<EventPump>> {
        self.event_pump.lock()
    }

    pub fn tff(&self) -> LockResult<MutexGuard<Sdl2TtfContext>> {
        self.tff_context.lock()
    }
}