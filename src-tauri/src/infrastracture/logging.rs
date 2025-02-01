use futures::stream::StreamExt;
use crate::constants::PATH_LOGGING;
use sentry_tracing::EventFilter;
use std::io::Write;
use tracing::Level;
use tracing_core::{Event, Subscriber};
use tracing_subscriber::fmt::{
    format::{self, FormatEvent, FormatFields},
    FmtContext, FormattedFields,
};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

struct MyFormatter;
// MyFormatter
impl<S, N> FormatEvent<S, N> for MyFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        // Format values from the event's's metadata:
        let metadata = event.metadata();
        write!(
            &mut writer,
            "{:<5} {} {} [{}({})] ",
            metadata.level(),
            thread_id::get(),
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f%Z"),
            metadata.target(),
            metadata.line().unwrap_or_default()
        )?;

        // Format all the spans in the event's span context.
        if let Some(scope) = ctx.event_scope() {
            for span in scope.from_root() {
                write!(writer, "{}", span.name())?;

                // `FormattedFields` is a formatted representation of the span's
                // fields, which is stored in its extensions by the `fmt` layer's
                // `new_span` method. The fields will have been formatted
                // by the same field formatter that's provided to the event
                // formatter in the `FmtContext`.
                let ext = span.extensions();
                let fields = &ext
                    .get::<FormattedFields<N>>()
                    .expect("will never be `None`");

                // Skip formatting the fields if the span had no fields.
                if !fields.is_empty() {
                    write!(writer, "{{{}}}", fields)?;
                }
                write!(writer, ": ")?;
            }
        }

        // Write fields on the event
        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}

pub fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    let binary_file_name = std::env::current_exe()
        .unwrap()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let path = PATH_LOGGING.as_str();
    // println!("path:{:?}", path);
    let file_appender = tracing_appender::rolling::daily(path, binary_file_name + ".log");

    // install global collector configured based on RUST_LOG env var.
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    //Errorレベルのeventのみ、sentryに送信
    let sentry_layer = sentry_tracing::layer().event_filter(|md| match md.level() {
        &tracing::Level::ERROR => EventFilter::Event,
        _ => EventFilter::Ignore,
    });

    //debug時はdebug情報を追加し、stdoutにも出力
    if cfg!(debug_assertions) {
        let subscriber = tracing_subscriber::registry()
            .with(EnvFilter::from_default_env().add_directive(Level::DEBUG.into()))
            // .with(sentry_layer) //debug時はsentryに送信しない
            .with(
                tracing_subscriber::fmt::Layer::new()
                    .compact()
                    .with_writer(std::io::stdout),
            )
            .with(
                tracing_subscriber::fmt::Layer::new()
                    .event_format(MyFormatter)
                    .with_writer(non_blocking),
            );
        // .with(tracing_subscriber::fmt::Layer::new().event_format(MyFormatter));

        tracing::subscriber::set_global_default(subscriber)
            .expect("Unable to set a global subscriber");
    } else {
        let subscriber = tracing_subscriber::registry()
            .with(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
            .with(sentry_layer)
            // .with(sentry_tracing::layer())
            .with(
                tracing_subscriber::fmt::Layer::new()
                    .event_format(MyFormatter)
                    .with_writer(non_blocking),
            );
        // .with(tracing_subscriber::fmt::Layer::new().event_format(MyFormatter));
        tracing::subscriber::set_global_default(subscriber)
            .expect("Unable to set a global subscriber");
    }
    _guard

    // let collector = tracing_subscriber::fmt()
    // .compact()
    // // filter spans/events with level TRACE or higher.
    // .with_max_level(if cfg!(debug_assertions) {Level::DEBUG} else {Level::INFO})
    // // build but do not install the subscriber.
    // .finish();
    // tracing::collect::with_default(collector, || {
    //     info!("This will be logged to stdout");
    // });
    // info!("This will _not_ be logged to stdout");

    // tracing_subscriber::fmt().compact().with_max_level(if cfg!(debug_assertions) {Level::DEBUG} else {Level::INFO}).with_writer(non_blocking).init();

    // tracing_subscriber::util::SubscriberInitExt
}
