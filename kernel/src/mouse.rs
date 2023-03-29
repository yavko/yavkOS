use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::sync::Arc;
use conquer_once::spin::OnceCell;
use crossbeam_utils::atomic::AtomicCell;

use crate::println;
use ps2_mouse::{Mouse as MouseDevice, MouseState};
use spinning_top::Spinlock;

static MOUSE: OnceCell<Mouse> = OnceCell::uninit();

pub fn init() {
    MOUSE.init_once(Mouse::default);
}

pub(crate) fn get() -> Option<Mouse> {
    MOUSE.get().cloned()
}

#[derive(Clone)]
pub(crate) struct Mouse {
    dev: Arc<Spinlock<MouseDevice>>,
    state: Arc<AtomicCell<Option<MouseState>>>,
    x: Arc<AtomicUsize>,
    y: Arc<AtomicUsize>,
}
impl Default for Mouse {
    fn default() -> Self {
        let mut cmd = x86_64::instructions::port::Port::<u8>::new(0x64);
        let mut data = x86_64::instructions::port::Port::<u8>::new(0x60);
        unsafe {
            cmd.write(0xa8); // enable aux port
            cmd.write(0x20); // read command byte
            let mut status = data.read();
            status |= 0b10; // enable aux port interrupts
            cmd.write(0x60); // write command byte
            data.write(status);
            cmd.write(0xd4); // signal that next write is to mouse input buffer
            data.write(0xf4); // enable mouse
        }

        let mut dev = MouseDevice::default();
        dev.set_on_complete(Self::handler);

        Self {
            dev: Arc::new(Spinlock::new(dev)),
            state: Default::default(),
            x: Default::default(),
            y: Default::default(),
        }
    }
}
//static LAST_POS: Spinlock<(AtomicUsize, AtomicUsize)> =
//    Spinlock::new((AtomicUsize::new(0), AtomicUsize::new(0)));
impl Mouse {
    fn handler(state: MouseState) {
        let this: &Mouse = MOUSE.get().expect("mouse not initialized");
        this.state.store(Some(state));

        this.set_pos();

        let x = this.x.load(Ordering::Relaxed);
        let y = this.y.load(Ordering::Relaxed);
        //let old = LAST_POS.lock();
        // crate::framebuffer::FRAMEBUFFER
        //     .get()
        //     .unwrap()
        //     .lock()
        //     .write_pixel(
        //         old.0.load(Ordering::Relaxed),
        //         old.1.load(Ordering::Relaxed),
        //         0,
        //     );
        crate::framebuffer::FRAMEBUFFER
            .get()
            .unwrap()
            .lock()
            .write_pixel(x, y, 255);
        // LAST_POS.lock().0.store(x, Ordering::Relaxed);
        // LAST_POS.lock().1.store(y, Ordering::Relaxed);
    }

    pub async fn add(&self, packet: u8) {
        let mut dev = self.dev.lock();
        dev.process_packet(packet);
    }

    fn set_pos(&self) {
        let state = self.state.load().expect("mouse state not initialized");
        let dx = state.get_x();
        let dy = state.get_y();

        if dx > 0 {
            self.x.fetch_add(dx as usize, Ordering::Relaxed);
        } else {
            self.x
                .fetch_sub(dx.unsigned_abs() as usize, Ordering::Relaxed);
        }

        if dy > 0 {
            self.y.fetch_sub(dy as usize, Ordering::Relaxed);
        } else {
            self.y
                .fetch_add(dy.unsigned_abs() as usize, Ordering::Relaxed);
        }

        self.limit_pos();
    }

    fn limit_pos(&self) {
        let x = self.x.load(Ordering::Relaxed);
        let y = self.y.load(Ordering::Relaxed);
        let (width, height) = crate::framebuffer::FRAMEBUFFER.get().unwrap().lock().size();

        if x > width - 1 {
            self.x.store(width - 1, Ordering::Relaxed);
        }

        if y > height - 1 {
            self.y.store(height - 1, Ordering::Relaxed);
        }
    }
}
