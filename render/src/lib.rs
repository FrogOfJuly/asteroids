use cursive::{View, view::Selector};
use std::sync::Mutex;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::HtmlCanvasElement;

mod backend;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub struct Cursive {
    _backend: Mutex<cursive::Cursive>,
}

pub struct Asteroids {}

impl Asteroids {
    pub fn new() -> Self {
        Asteroids {}
    }
}

impl Default for Asteroids {
    fn default() -> Self {
        Self::new()
    }
}

impl View for Asteroids {
    fn draw(&self, printer: &cursive::Printer) {
        printer.theme(&cursive::theme::Theme::retro());
    }
}

#[wasm_bindgen]
impl Cursive {
    #[wasm_bindgen(js_name = "asteroids")]
    pub async fn asteroids() -> Cursive {
        set_panic_hook();
        alert("Hello!");
        let mut siv: cursive::Cursive = cursive::Cursive::new();
        let asteroids = Asteroids::new(); //.with_name("asteroids");
        siv.add_layer(asteroids);
        siv.focus(&Selector::Name("asteroids")).unwrap();
        siv.set_fps(1000);
        let siv: Mutex<cursive::Cursive> = std::sync::Mutex::new(siv);
        siv.lock().unwrap().run_with(|| backend::backend()).await;
        Cursive { _backend: siv }
    }

    #[wasm_bindgen(js_name = "asteroids_with_canvas")]
    pub async fn asteroids_with_canvas(canvas: HtmlCanvasElement) -> Cursive {
        set_panic_hook();
        alert("Hello!");
        let mut siv: cursive::Cursive = cursive::Cursive::new();
        let asteroids = Asteroids::new(); //.with_name("asteroids");
        siv.add_layer(asteroids);
        siv.focus(&Selector::Name("asteroids")).unwrap();
        siv.set_fps(1000);
        let siv: Mutex<cursive::Cursive> = std::sync::Mutex::new(siv);
        siv.lock()
            .unwrap()
            .run_with(|| backend::backend_with_canvas(canvas))
            .await;
        Cursive { _backend: siv }
    }
}
