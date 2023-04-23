//!A wrapper around [minifb] that makes managing windows as simple as possible, with hexidecimal RGB.
//!
//!Window elements like buffers and dimensions are linked together in the [WindowContainer] struct. To keep the window alive, call update() from within a loop. This has a built-in graceful exit by pressing ESC.
//!
//!```rust
//!let window = window!(?500, 500, "Window", "FFFFFF");
//!
//!loop {
//!    window.update();
//!}
//!```
//! 
//!Note with the hexidecimal conversions: functions that take hexidecimal will take a &str for convenience, but functions that return hexidecimal will return a [String].
//! 














//use minifb;
use std::{process, ops::Range};
fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}

type Unit = Result<(), WinErr>;

macro_rules! unit {
    () => {
        Ok(())
    };
}
///Initializes a [WindowContainer]. Optional background color. Begin with ? to unwrap.
#[macro_export] macro_rules! window {
    ($width:expr, $height:expr, $name:tt) => {
        WindowContainer::new($width, $height, $name, "FFFFFF")
    };
    (?$width:expr, $height:expr, $name:tt) => {
        WindowContainer::new($width, $height, $name, "FFFFFF").unwrap()
    };

    ($width:expr, $height:expr, $name:tt, $color:tt) => {
        WindowContainer::new($width, $height, $name, $color)
    };
    (?$width:expr, $height:expr, $name:tt, $color:tt) => {
        WindowContainer::new($width, $height, $name, $color).unwrap()
    };

}

///converts a 0F0F0F format hex string into a u32.
pub fn hex_to_rgb(code:String) -> Result<u32, WinErr> {
    if code.len() != 6 {return Err(WinErr::InvalidHexCode(code))}

    let (r, gb) = code.split_at(2);
    let (g, b) = gb.split_at(2);
    
    let out = from_u8_rgb(
    from_hex(r.to_string())?,
    from_hex(g.to_string())?,
    from_hex(b.to_string())?
    );

    Ok(out)
}



fn hex2num_code(c: char) -> Result<u8, WinErr> {
    c.to_digit(16)
        .map(|n| n as u8)
        .ok_or(WinErr::InvalidHexChar(c))
}
///Converts a hexidecimal string to a u8.
pub fn from_hex(input:String) -> Result<u8, WinErr> {
    let mut output = 0_u8;
    
    let iterator = input.chars().rev().enumerate();

    for (i, c) in iterator {
        let val = hex2num_code(c)?;
        output += val * 16_u8.pow(i as u32);
    }

    Ok(output)
}
fn num2hex_code(num: u8) -> Result<char, WinErr> {
    match num {
        0..=9 => Ok((num + 48) as char),
        10..=15 => Ok((num + 55) as char),
        _ => Err(WinErr::InvalidNumCode(num))
    }
}
///Converts a u32 to a hexidecimal string. Note that this includes leading zeros.
pub fn to_hex(num: u32) -> Result<String, WinErr> {
    let mut hex_string = String::new();

    for i in (0..8).rev() {
        let shift = i * 4;
        let hex_digit = ((num >> shift) & 0xF) as u8;
        let hex_char = num2hex_code(hex_digit)?;

        hex_string.push(hex_char);
    }

    Ok(hex_string)
}
///Converts a u8 to a 2-char wide hexidecimal string, to concatenate into a RGB hex string. Note that this includes leading zero.
pub fn to_hex_u8(num: u8) -> Result<String, WinErr> {
    let mut hex_string = String::new();

    for i in (0..2).rev() {
        let shift = i * 4;
        let hex_digit = (num >> shift) & 0xF;
        let hex_char = num2hex_code(hex_digit)?;

        hex_string.push(hex_char);
    }

    Ok(hex_string)
}

fn dist(p1:(usize, usize), p2:(usize, usize)) -> f64 {
    let fp1 = (p1.0 as f64, p1.1 as f64);
    let fp2 = (p2.0 as f64, p2.1 as f64);
    ((fp2.0 - fp1.0).powi(2) + (fp2.1 - fp1.1).powi(2)).sqrt()

}

fn tupf64(tup:(usize, usize)) -> (f64, f64) {
    (tup.0 as f64, tup.1 as f64)
}

fn perpendicular_line(slope: f64, midpoint: (f64, f64), length: f64) -> ((f64, f64), (f64, f64)) {
    let m_perp = -1.0 / slope;
    let dx = length / (2.0 * (1.0 + m_perp.powi(2)).sqrt());
    let dy = m_perp * dx;
    let start = (midpoint.0 - dx, midpoint.1 - dy);
    let end = (midpoint.0 + dx, midpoint.1 + dy);
    (start, end)
}

///Contains a Window and pixel buffer, and properties.
pub struct WindowContainer {
    buffer:Vec<u32>,
    window:minifb::Window,

    width:usize,
    height:usize,
    pub bg_color:u32,

    length:usize,

}

impl WindowContainer {
    ///Initalizes a new WindowContainer.
    pub fn new(width:usize, height:usize, name:&str, color:&str) -> Result<Self, WinErr> {
        let bg_color = hex_to_rgb(color.to_string())?;
        if bg_color > 16777215 {return Err(WinErr::InvalidRGBValue(bg_color))}

        let buffer = vec![bg_color; width * height];
        let window = minifb::Window::new(name, width, height, minifb::WindowOptions::default()).unwrap();

        let length = buffer.len();

        Ok(WindowContainer {buffer, window, width, height, bg_color, length})
    }

    ///Updates the window with its pixel buffer. Pressing ESC will gracefully exit the program.
    pub fn update(&mut self) -> Unit {
        if self.window.is_key_down(minifb::Key::Escape) {process::exit(1)}


        self.window.update_with_buffer(&self.buffer, self.width, self.height)?;
        unit!()
    }
    ///clears the screen to the background color.
    pub fn clear(&mut self) {
        self.buffer = self.buffer.iter().map(|_| self.bg_color).collect();
    }

    //inputs should be converted from &str -> String so that inputing params is easier
    ///Returns the hexidecimal value of a pixel at a position.
    pub fn get(&self, pos:(usize, usize)) -> Result<String, WinErr> {
        if pos.0 >= self.width  {return Err(WinErr::InvalidPos(pos))}
        if pos.1 >= self.height {return Err(WinErr::InvalidPos(pos))}

        let i = (pos.1 * self.width) + pos.0;
        let val = self.buffer[i];
        
        to_hex(val)
    }
    ///Sets a pixel to a hexidecimal value.
    pub fn set(&mut self, pos:(usize, usize), val:&str) -> Result<(), WinErr> {
        if pos.0 >= self.width  {return Err(WinErr::InvalidPos(pos))}
        if pos.1 >= self.height {return Err(WinErr::InvalidPos(pos))}

        let i = (pos.1 * self.width) + pos.0;
        self.buffer[i] = hex_to_rgb(val.to_string())?;

        unit!()
    }

    ///returns an iterator over the pixel buffer, holding the raw u32 value and position. Note that the iterator pulled from the buffer is no longer linked to the window, and modifying it will do nothing.
    pub fn iter(&self) -> std::vec::IntoIter<(u32, (usize, usize))> {
        let mut table:Vec<(u32, (usize, usize))> = vec![];
        
        for i in 0..self.length {
            table.push((self.buffer[i], self.nth_to_pos(i)));
        }

        table.into_iter()
    }


    fn pos_to_nth(&self, pos:(usize, usize)) -> usize {
        (pos.1 * self.width) + pos.0
    }
    fn nth_to_pos(&self, i:usize) -> (usize, usize) {
        (i / self.width, i % self.width)
    }

    //drawing
    ///Draws a circle at a given position with a radius and color.
    pub fn circle(&mut self, pos:(usize, usize), r:usize, color:&str) -> Unit {
        if pos.0 >= self.width  {return Err(WinErr::InvalidPos(pos))}
        if pos.1 >= self.height {return Err(WinErr::InvalidPos(pos))}

        let y_range = pos.1-r..pos.1+r;

        let mut range_table: Vec<Range<usize>> = vec![];
        
        for i in y_range {
            range_table.push((pos.0+(self.width*i))-r..(pos.0+(self.width*i))+r)
        }

        for range in range_table {
            for i in range {
                let loc = self.nth_to_pos(i);

                if dist(loc, pos) < r as f64 {self.buffer[i] = hex_to_rgb(color.to_string())?}
            }
        }


        unit!()
    }
    ///Draws a line between 2 points, with a thickness and color.
    pub fn line(&mut self, p1:(usize, usize), p2:(usize, usize), t:f64, color:&str) -> Unit {
        let fp1 = tupf64(p1);
        let fp2 = tupf64(p2);

        //let normal_p1 = (0.0, 0.0);
        let normal = (fp2.0-fp1.0, fp2.1-fp1.1);
        
        let m = (fp2.1 - fp1.1) / (fp2.0 - fp1.0);
        let p_line = perpendicular_line(m, fp1, t);
        let p_normal = (p_line.1.0 - p_line.0.0, p_line.1.1 - p_line.0.1);

        //this should make the step smaller the longer the line is
        let step = (1.0/dist(p1, p2)) * 0.9999;
        let p_step = 0.5/t;
        let mut t = 0.0;
        println!("{step}");
            while t <= 1.0 {
                let point = ((normal.0*t)+fp1.0, (normal.1*t)+fp1.1);
                let usize_p = (point.0 as usize, point.1 as usize);

                let loc = self.pos_to_nth(usize_p);
                

                if usize_p.0 < self.width && usize_p.1 < self.height {
                    self.buffer[loc] = hex_to_rgb(color.to_string())?;
                }

                let p_line = perpendicular_line(m, point, t);

                let mut j = 0.0;
                while j <= 1.0 {
                    let p_point = ((p_normal.0*j) + p_line.0.0, (p_normal.1*j) + p_line.0.1);
                    let usize_p_point = (p_point.0 as usize, p_point.1 as usize);

                    let p_loc = self.pos_to_nth(usize_p_point);
                    
                    if usize_p_point.0 < self.width && usize_p_point.1 < self.height {
                        self.buffer[p_loc] = hex_to_rgb(color.to_string())?;
                    }

                    j += p_step;
                }

                t += step;
            }


        unit!()
    }

}



























#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        //const escape = UpdateOptions::escape;
        let mut window = window!(?255, 255, "Window", "FF00FF");
        loop {
            window.update();
        }
    }
    #[test]
    fn uv_map() -> Unit {
        let mut window = WindowContainer::new(255, 255, "Window", "FFFFFF").unwrap();

        for (_val, pos) in window.iter() {
            let r = to_hex_u8(pos.0 as u8).unwrap();
            let g = to_hex_u8(pos.1 as u8).unwrap();
            let string = format!("{r}{g}00");
            window.set(pos, &string);
        }
        
        loop {
            window.update();
        }

        unit!()
    }
    
    #[test]
    fn shapes() -> Unit {
        let mut window = WindowContainer::new(1000, 1000, "Window", "FFFFFF").unwrap();

        window.circle((100, 100), 50, "CC00FF");
        
        window.line((800, 1000), (800, 0), 50.0, "FF0000");
        window.line((200, 200), (150, 230), 10.0, "0000FF");
        //window.line((10, 170), (3000, 170), 3.0, "FF00FF");
        
        loop {
            window.update();
        }

        unit!()
    }
}

///Errors concerning the WindowContainer.
#[derive(Debug)]
pub enum WinErr {
    MinifbError(minifb::Error),

    InvalidHexCode(String),
    InvalidHexChar(char),
    InvalidNumCode(u8),
    InvalidRGBValue(u32),

    InvalidIndex(usize),
    InvalidPos((usize, usize)),
}

impl From<minifb::Error> for WinErr {
    fn from(cause:minifb::Error) -> Self {
        WinErr::MinifbError(cause)
    }
}

