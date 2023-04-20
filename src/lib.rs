use minifb;
use std::process;
fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}

type unit = Result<(), WinErr>;

macro_rules! unit {
    () => {
        Ok(())
    };
}


pub fn hex_to_rgb(code:&str) -> Result<u32, WinErr> {
    if code.len() != 6 {return Err(WinErr::InvalidHexCode(String::from(code)))}

    let (r, gb) = code.split_at(2);
    let (g, b) = gb.split_at(2);
    
    let out = from_u8_rgb(
    from_hex(r)?,
    from_hex(g)?,
    from_hex(b)?
    );

    Ok(out)
}



fn hex2num_code(c: char) -> Result<u8, WinErr> {
    c.to_digit(16)
        .map(|n| n as u8)
        .ok_or(WinErr::InvalidHexChar(c))
}
fn from_hex(input:&str) -> Result<u8, WinErr> {
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
fn to_hex(num: u32) -> Result<String, WinErr> {
    let mut hex_string = String::new();

    for i in (0..8).rev() {
        let shift = i * 4;
        let hex_digit = ((num >> shift) & 0xF) as u8;
        let hex_char = num2hex_code(hex_digit)?;

        hex_string.push(hex_char);
    }

    Ok(hex_string)
}

pub struct WindowContainer {
    pub buffer:Vec<u32>,
    pub window:minifb::Window,

    pub width:usize,
    pub height:usize,
    pub bg_color:u32,

    length:usize,
}

impl WindowContainer {
    pub fn new_dec(width:usize, height:usize, name:&str, bg_color:u32) -> Result<Self, WinErr> {
        if bg_color > 16777215 {return Err(WinErr::InvalidRGBValue(bg_color))}
        
        let buffer = vec![bg_color; width * height];
        let window = minifb::Window::new(name, width, height, minifb::WindowOptions::default()).unwrap();

        let length = buffer.len();

        Ok(WindowContainer {buffer, window, width, height, bg_color, length})
    }

    pub fn new_hex(width:usize, height:usize, name:&str, color:&str) -> Result<Self, WinErr> {
        let bg_color = hex_to_rgb(color)?;
        if bg_color > 16777215 {return Err(WinErr::InvalidRGBValue(bg_color))}

        let buffer = vec![bg_color; width * height];
        let window = minifb::Window::new(name, width, height, minifb::WindowOptions::default()).unwrap();

        let length = buffer.len();

        Ok(WindowContainer {buffer, window, width, height, bg_color, length})
    }


    pub fn update(&mut self) -> unit {
        if self.window.is_key_down(minifb::Key::Escape) {process::exit(1)}


        self.window.update_with_buffer(&self.buffer, self.width, self.height)?;
        unit!()
    }

    pub fn clear(&mut self) {
        self.buffer = self.buffer.iter().map(|_| self.bg_color).collect();
    }


    pub fn nth(&mut self, i:usize) -> Result<WinElement, WinErr> {
        if i >= self.length {return Err(WinErr::InvalidIndex(i))}

        Ok(WinElement {value:self.buffer[i].clone(), owner:self, index:i})
    }

    pub fn pos(&mut self, pos:(usize, usize)) -> Result<WinElement, WinErr> {
        if pos.0 >= self.width  {return Err(WinErr::InvalidPos(pos))}
        if pos.1 >= self.height {return Err(WinErr::InvalidPos(pos))}

        let i = (pos.1 * self.width) + pos.0;

        Ok(WinElement {value:self.buffer[i].clone(), owner:self, index:i})
    }

}

pub struct WinElement<'a> {
    owner:&'a mut WindowContainer,
    value:u32,
    index:usize,
}
//consumes self after use
impl WinElement<'_> {
    pub fn read_hex(self) -> String {
        //it should not be possible for this to throw
        to_hex(self.value).unwrap()
    }

    pub fn read_dec(self) -> u32 {
        self.value
    }

    pub fn write_hex(self, value:&str) -> Result<(), WinErr> {
        self.owner.buffer[self.index] = hex_to_rgb(value)?;

        unit!()
    }
    pub fn write_dec(self, value:u32) -> Result<(), WinErr> {
        if value > 16777215 {return Err(WinErr::InvalidRGBValue(value))}
        self.owner.buffer[self.index] = value;

        unit!()
    }


}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        //const escape = UpdateOptions::escape;
        let mut window = WindowContainer::new_hex(255, 255, "Window", "FFFFFF").unwrap();
        loop {
            window.update();
        }
    }
    #[test]
    fn rw() -> unit {
        let mut window = WindowContainer::new_hex(255, 255, "Window", "FFFFFF").unwrap();

        for i in 0..window.width * window.height {window.nth(i)?.write_hex("CC00FF");}



        let val = window.nth(15)?.read_hex();
        println!("{val}");

        loop {
            window.update();
        }

        unit!()
    }
}

#[derive(Debug)]
pub enum WinErr {
    minifbError(minifb::Error),

    InvalidHexCode(String),
    InvalidHexChar(char),
    InvalidNumCode(u8),
    InvalidRGBValue(u32),

    InvalidIndex(usize),
    InvalidPos((usize, usize)),
}

impl From<minifb::Error> for WinErr {
    fn from(cause:minifb::Error) -> Self {
        WinErr::minifbError(cause)
    }
}

//general input conf
