//! macOS: the system's "fonts changed" signal. CoreText posts
//! `kCTFontManagerRegisteredFontsChangedNotification` when fonts are installed, removed, or
//! activated or deactivated by a font manager or Font Book
//! (<https://developer.apple.com/documentation/coretext/kctfontmanagerregisteredfontschangednotification>).
//! The app then looks at its fonts at once instead of waiting for its timer or for the window to
//! come back, so the font lists follow live.
//!
//! CoreText delivers it on the app's own (local) notification center: a font manager switching
//! fonts on or off at the persistent scope reaches this process there, not on the distributed
//! center. Both are listened to.
//!
//! A font manager switching a family on posts the signal several times in a row (once per batch
//! CoreText registers), so the look waits until the signals have stopped for [`SETTLE`]: one
//! rescan per batch.
#![allow(unsafe_code)]

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Once, OnceLock};
use std::time::{Duration, Instant};

use block2::RcBlock;
use objc2_core_text::kCTFontManagerRegisteredFontsChangedNotification;
use objc2_foundation::{NSDistributedNotificationCenter, NSNotification, NSNotificationCenter, NSString};

/// How long the signals must have stopped before the fonts are looked at.
const SETTLE: Duration = Duration::from_millis(500);

/// A signal came that [`take`] hasn't answered yet.
static PENDING: AtomicBool = AtomicBool::new(false);
/// When the last signal came, in milliseconds since [`epoch`].
static LAST_MS: AtomicU64 = AtomicU64::new(0);

fn epoch() -> Instant {
    static EPOCH: OnceLock<Instant> = OnceLock::new();
    *EPOCH.get_or_init(Instant::now)
}

fn now_ms() -> u64 {
    u64::try_from(epoch().elapsed().as_millis()).unwrap_or(u64::MAX)
}

/// Listen for the signal; each one wakes the app (`ctx`) once the batch has settled.
pub fn install(ctx: &egui::Context) {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        epoch();
        // SAFETY: CoreText initializes this immutable notification-name constant.
        let name = NSString::from_str(&unsafe { kCTFontManagerRegisteredFontsChangedNotification }.to_string());
        let repaint = ctx.clone();
        // Runs on whichever thread posted the signal: it only touches atomics and asks for a
        // repaint, both safe from any thread.
        let block = RcBlock::new(move |_: std::ptr::NonNull<NSNotification>| {
            log::debug!("fonts changed signal");
            LAST_MS.store(now_ms(), Ordering::Relaxed);
            PENDING.store(true, Ordering::Release);
            // Wake the app when this batch has settled (a later signal pushes it back).
            repaint.request_repaint_after(SETTLE);
        });
        // SAFETY: each notification center retains its observer and the block for the
        // application's lifetime; the block only sets atomics and asks for a repaint.
        unsafe {
            NSNotificationCenter::defaultCenter().addObserverForName_object_queue_usingBlock(Some(&name), None, None, &block);
            NSDistributedNotificationCenter::defaultCenter().addObserverForName_object_queue_usingBlock(Some(&name), None, None, &block);
        }
    });
}

/// Whether signals came and have since settled for [`SETTLE`] (then they count as answered).
/// While a batch is still arriving, asks `ctx` to look again when it may have settled.
pub fn take(ctx: &egui::Context) -> bool {
    if !PENDING.load(Ordering::Acquire) {
        return false;
    }
    let quiet = Duration::from_millis(now_ms().saturating_sub(LAST_MS.load(Ordering::Relaxed)));
    if quiet < SETTLE {
        ctx.request_repaint_after(SETTLE.saturating_sub(quiet));
        return false;
    }
    // A signal arriving right now sets it again and is answered on a later frame.
    let settled = PENDING.swap(false, Ordering::AcqRel);
    log::debug!("fonts changed signals settled: looking at the installed fonts");
    settled
}
