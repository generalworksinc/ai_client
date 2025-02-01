use crate::constants;

// pub fn init_error_handle(is_test: Option<bool>) -> sentry::ClientInitGuard {
//     std::env::set_var("RUST_BACKTRACE", "1");
//     let is_debug_mode = cfg!(debug_assertions);
//     sentry::init((
//         if is_debug_mode || is_test == Some(true) {
//             constants::SENTRY_URL_DEV
//         } else {
//             constants::SENTRY_URL
//         },
//         sentry::ClientOptions {
//             release: sentry::release_name!(),
//             ..Default::default()
//         },
//     ))
// }
