#![cfg(any(target_os = "macos", target_os = "ios"))]

mod ffi {
    #![allow(non_upper_case_globals, non_camel_case_types, dead_code)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use std::{
    ffi::{CStr, CString},
    io,
};

use tracing_core::{
    span::{Attributes, Id, Record},
    Event, Level, Metadata, Subscriber,
};
use tracing_subscriber::{
    fmt::{
        format::{DefaultFields, Format, Full},
        Layer as FmtLayer, MakeWriter,
    },
    layer::Context,
    registry::LookupSpan,
    Layer,
};

use crate::ffi::{
    os_log_create, os_log_t, os_log_type_t_OS_LOG_TYPE_DEBUG, os_log_type_t_OS_LOG_TYPE_ERROR,
    os_log_type_t_OS_LOG_TYPE_INFO, os_release, wrapped_os_log_with_type,
};

struct OsLog(os_log_t);

impl OsLog {
    fn new(subsystem: &CStr, category: &CStr) -> Self {
        let logger = unsafe { os_log_create(subsystem.as_ptr(), category.as_ptr()) };
        Self(logger)
    }
}

impl Drop for OsLog {
    fn drop(&mut self) {
        unsafe { os_release(self.0 as *mut _) };
    }
}

pub struct OsLogLayer<S> {
    fmt_layer: FmtLayer<S, DefaultFields, Format<Full, ()>, OsLogMakeWriter>,
}

unsafe impl<S> Sync for OsLogLayer<S> {}
unsafe impl<S> Send for OsLogLayer<S> {}

impl<S> OsLogLayer<S> {
    /// Initialize a new `OsLogger`, which will output [tracing] events to os_log on Apple platforms.
    ///
    /// # Arguments
    ///
    /// * `subsystem` - An identifier string, in reverse DNS notation, that represents the subsystem that’s performing logging, for example, `com.your_company.your_subsystem_name`. The subsystem is used for categorization and filtering of related log messages, as well as for grouping related logging settings.
    /// * `category` - A category within the specified subsystem. The system uses the category to categorize and filter related log messages, as well as to group related logging settings within the subsystem’s settings. A category’s logging settings override those of the parent subsystem.
    pub fn new(subsystem: &CStr, category: &CStr) -> Self {
        let logger = OsLog::new(subsystem, category);

        let fmt_layer = FmtLayer::new()
            .without_time()
            .with_level(false)
            .with_ansi(false)
            .with_target(false)
            .with_writer(OsLogMakeWriter::new(logger.0));

        Self { fmt_layer }
    }
}

macro_rules! impl_layer {
    ($(fn $method:ident(&self $(, $arg_name:ident: $arg_type:ty)*) $(-> $return_type:ty)?;)*) => {
        $(
            #[inline]
            fn $method(&self $(, $arg_name: $arg_type)*) $(-> $return_type)? {
                self.fmt_layer.$method($($arg_name),*)
            }
        )*
    };
}

impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for OsLogLayer<S> {
    impl_layer!(
        fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>);
        fn on_record(&self, span: &Id, values: &Record<'_>, ctx: Context<'_, S>);
        fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>);
        fn on_enter(&self, id: &Id, ctx: Context<'_, S>);
        fn on_exit(&self, id: &Id, ctx: Context<'_, S>);
        fn on_close(&self, id: Id, ctx: Context<'_, S>);
    );
}

struct OsLogMakeWriter {
    logger: os_log_t,
}

impl OsLogMakeWriter {
    fn new(logger: os_log_t) -> Self {
        Self { logger }
    }

    fn make_writer(&self, level: Level) -> OsLogWriter {
        OsLogWriter {
            logger: self.logger,
            level,
        }
    }
}

impl<'a> MakeWriter<'a> for OsLogMakeWriter {
    type Writer = OsLogWriter;

    fn make_writer(&'a self) -> Self::Writer {
        // `MakeWriter::make_writer()` is not called since `MakeWriter::make_writer_for()` is defined
        self.make_writer(Level::INFO)
    }

    fn make_writer_for(&'a self, meta: &Metadata<'_>) -> Self::Writer {
        self.make_writer(*meta.level())
    }
}

struct OsLogWriter {
    logger: os_log_t,
    level: Level,
}

impl OsLogWriter {
    fn log_event(&self, buf: &[u8]) -> io::Result<usize> {
        let level = match self.level {
            Level::TRACE => os_log_type_t_OS_LOG_TYPE_DEBUG,
            Level::DEBUG => os_log_type_t_OS_LOG_TYPE_DEBUG,
            Level::INFO => os_log_type_t_OS_LOG_TYPE_INFO,
            Level::WARN => os_log_type_t_OS_LOG_TYPE_ERROR,
            Level::ERROR => os_log_type_t_OS_LOG_TYPE_ERROR,
        };

        let msg = CString::new(buf).map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;

        unsafe {
            wrapped_os_log_with_type(self.logger, level, msg.as_ptr());
        }

        Ok(buf.len())
    }
}

impl io::Write for OsLogWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.log_event(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.log_event(buf).map(|_| ())
    }
}
