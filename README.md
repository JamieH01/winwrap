A wrapper around minifb that makes opening and managing windows as straightforward as possible, and with hexidecimal RGB rather than raw u32.

This is an example code that will generate a UV coordinate map:
```rust
let window = window!(?500, 500, "Window", "FFFFFF");

//iterates with the value and position
for (_, pos) in window.iter() {
    let r = to_hex2(pos.0 as u8).unwrap();
    let g = to_hex2(pos.1 as u8).unwrap();
    let string = format!("{r}{g}00");

    window.set(pos, &string)?;
}

//pressing escape will close the window
loop {
    window.update();
}
```
