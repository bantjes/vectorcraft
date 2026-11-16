//! macOS: the system's "fonts changed" signal. CoreText posts
//! `kCTFontManagerRegisteredFontsChangedNotification` on the distributed notification center when
//! fonts are installed, removed, or activated or deactivated by a font manager
//! (<https://developer.apple.com/documentation/coretext/kctfontmanagerregisteredfontschangednotification>).
//! The app then looks at its fonts at once instead of waiting for its timer, so the font lists
//! follow live.
#![allow(unsafe_code)]

use std::sync::Once;
use std::sync::atomic::{AtomicBool, Ordering};

use block2::RcBlock;
use objc2_core_text::kCTFontManagerRegisteredFontsChangedNotification;
use objc2_foundation::{NSDistributedNotificationCenter, NSNotification, NSString};

static CHANGED: AtomicBool = AtomicBool::new(false);

/// Listen for the signal; each one wakes the app (`ctx`).
pub fn install(ctx: &egui::Context) {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        // SAFETY: CoreText initializes this immutable notification-name constant.
        let name = NSString::from_str(&unsafe { kCTFontManagerRegisteredFontsChangedNotification }.to_string());
        let repaint = ctx.clone();
        let block = RcBlock::new(move |_: std::ptr::NonNull<NSNotification>| {
            CHANGED.store(true, Ordering::Relaxed);
            repaint.request_repaint();
        });
        // SAFETY: the notification center retains the observer and the block for the application's
        // lifetime; the block only sets an atomic and asks for a repaint.
        unsafe {
            NSDistributedNotificationCenter::defaultCenter().addObserverForName_object_queue_usingBlock(Some(&name), None, None, &block);
        }
    });
}

/// Whether the signal came since the last call.
pub fn take() -> bool {
    CHANGED.swap(false, Ordering::Relaxed)
}
