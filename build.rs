use std::env;

fn main() {
    if let Ok(pthreads_win) = env::var("WIN_PTHREADS") {
        println!("cargo:rustc-link-search=native={}", pthreads_win);
    }
}
