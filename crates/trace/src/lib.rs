//! Pixel helpers left over from Image Trace, which this build doesn't have: the outlines of a pixel
//! mask (opening legacy Illustrator files) and the tile colours of Object › Create Object Mosaic.

#![forbid(unsafe_code)]

mod contour;
mod mosaic;
mod raster;

pub use contour::{Component, Loop, trace_mask};
pub use mosaic::mosaic;
pub use raster::{MAX_PIXELS, MAX_SOURCE_PIXELS, Raster, TraceError};
