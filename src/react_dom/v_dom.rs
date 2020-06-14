use web_sys::window;


pub fn reactdom() {
    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.
    let window = window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    // Manufacture the element we're gonna append
    let val = document.create_element("p");
    match val {
        Ok(el) => {
            el.set_inner_html("Hello from Rust! bravo djedou");
            body.append_child(&el);
        },
        Err(value) => {
            println!("{:#?}", value);
        }
    }

}