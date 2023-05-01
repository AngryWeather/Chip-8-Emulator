extern crate sdl2;
use rand::Rng;
use rand::distributions::DistString;
use rand::thread_rng;
use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Canvas;
use sdl2::render::Texture;
use sdl2::sys::KeySym;
use sdl2::sys::SDL_QuitEvent;
use sdl2::video::Window;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::io;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::SeekFrom;
use std::io::Seek;
use std::ops::AddAssign;
use std::time::Instant;
use std::time::SystemTime;

fn main() -> io::Result<()>{

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("chip8-emulator",640, 320)
        .position_centered()
        .build()
        .unwrap();

        let mut event_pump = sdl_context.event_pump().unwrap();
        let mut canvas = window.into_canvas().build().unwrap();

        let creator = canvas.texture_creator();
        canvas.set_scale(10.0, 10.0).unwrap();
        let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 64, 32).unwrap();

        canvas.set_draw_color(sdl2::pixels::Color::BLACK);
        canvas.clear();

        let args: Vec<String> = env::args().collect();
        let file_path = &args[1];

        let f = File::open(file_path)?;
        let mut reader = BufReader::new(f);
        let mut buffer: Vec<u8> = Vec::new();
        
        let f_size = reader.seek(SeekFrom::End(0))?;
        reader.rewind()?;
        reader.read_to_end(&mut buffer)?;
        let mut font: [u8; 0x200] = [0; 0x200];
        
        // chip8-8 puts programs in memory at 0x200
        let mut chip8 = Chip8State::new([0;1024*4], [0; 64 * 32 * 3], 
            [0; 16], 0x0, 0x0, 0x0);    

        chip8.memory[0x200 .. (0x200 + &buffer.len())].copy_from_slice(&buffer[..]);
        chip8.font = [
            0xf0, 0x90, 0x90, 0x90, 0xf0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xf0, 0x10, 0xf0, 0x80, 0xf0, // 2
            0xf0, 0x10, 0xf0, 0x10, 0xf0, // 3
            0x90, 0x90, 0xf0, 0x10, 0x10, // 4
            0xf0, 0x80, 0xf0, 0x10, 0xf0, // 5
            0xf0, 0x80, 0xf0, 0x90, 0xf0, // 6
            0xf0, 0x10, 0x20, 0x40, 0x40, // 7
            0xf0, 0x90, 0xf0, 0x90, 0xf0, // 8
            0xf0, 0x90, 0xf0, 0x10, 0xf0, // 9
            0xf0, 0x90, 0xf0, 0x90, 0x90, // A
            0xe0, 0x90, 0xe0, 0x90, 0xe0, // B
            0xf0, 0x80, 0x80, 0x80, 0xf0, // C
            0xe0, 0x90, 0x90, 0x90, 0xe0, // D
            0xf0, 0x80, 0xf0, 0x80, 0xf0, // E
            0xf0, 0x80, 0xf0, 0x80, 0x80, // F
        ];

        chip8.memory[0x0 .. 0x50].copy_from_slice(&chip8.font);
        println!("BUFFER Len: {:x?}", &buffer.len());
        
    'running: loop {
        canvas.clear();
        // for event in event_pump.poll_iter() {
        //     // if event {  WindowEvent::Close};

            
        //     match event {
        //         Event::Quit{..} => WindowEvent::Close,
        //         _ => WindowEvent::Shown,
        //     };
        // }
        let delta = 0; 
        let mut accumulator: f32 = 0.0;
        const delay_time: f32 = 16.7;
        // let delay_in_nanos = 

        while (chip8.pc) < 0x200 + buffer.len() as u16{
            let now = SystemTime::now();
            let keys: HashSet<Scancode> = event_pump
                .keyboard_state()
                .pressed_scancodes()
                .collect();
            
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit {..} => break 'running,
                    _ => {},
                }
            }
            let last_time = now.elapsed().unwrap().as_nanos();
            println!("NANOS: {last_time}");
            // let mut i = last_time;
            // ::std::thread::sleep(std::time::Duration::new(0, last_time as u32));
            disassemble(&mut chip8, &mut canvas, &mut texture, &mut event_pump, &keys, &last_time);
            chip8.pc += 2;
            print!("\n"); 
            
            // std::thread::sleep(std::time::Duration::new(1, 0));
            println!("DELAY: {}", &chip8.delay);

            if chip8.delay > 0 {
                chip8.delay -= 1;
            } 
            
        }
    }

    Ok(())
}

struct Chip8State {
    v: [u8; 16],
    i: u16,
    sp: u16,
    pc: u16,
    delay: u8,
    font: [u8; 0x50],
    sound: u8,
    memory: [u8; 1024 * 4],
    screen: [u8; 64 * 32 * 3],
    stack: Vec<u16>,
}

impl Chip8State {
    fn new(memory: [u8; 1024 * 4], screen:  [u8; 64 * 32 * 3], 
        v: [u8;16], i: u16, delay: u8, sound: u8) -> Self {Chip8State {
            memory,
            screen,
            sp: 0,
            pc: 0x200,
            v,
            font: [0; 0x50],
            i: 0x0,
            delay,
            sound,
            stack: Vec::with_capacity(16),
        }
    }
}

fn get_codes(chip8_mem: [u8; 4096], pc: usize) -> (u8, u8) {
    let code0 = chip8_mem[pc];
    let code1 = chip8_mem[pc + 1];
    
    (code0, code1)
}

fn get_key_map() -> HashMap<Scancode, u8> {
    let mut key_map: HashMap<Scancode, u8> = HashMap::new();

    key_map.insert(Scancode::Num1, 0x1);
    key_map.insert(Scancode::Num2, 0x2);
    key_map.insert(Scancode::Num3, 0x3);
    key_map.insert(Scancode::Num4, 0xc);
    key_map.insert(Scancode::Q, 0x4);
    key_map.insert(Scancode::W, 0x5);
    key_map.insert(Scancode::E, 0x6);
    key_map.insert(Scancode::A, 0x7);
    key_map.insert(Scancode::S, 0x8);
    key_map.insert(Scancode::D, 0x9);
    key_map.insert(Scancode::R, 0xd);
    key_map.insert(Scancode::F, 0xe);
    key_map.insert(Scancode::Z, 0xa);
    key_map.insert(Scancode::X, 0x0);
    key_map.insert(Scancode::C, 0xb);
    key_map.insert(Scancode::V, 0xf);
    
    key_map
    
}


fn disassemble(chip8: &mut Chip8State, canvas: &mut Canvas<Window>, texture: &mut Texture, event_pump: &mut EventPump, keys: &HashSet<Scancode>, time: &u128) {
    ::std::thread::sleep(std::time::Duration::new(0, 1666667 as u32));

    let pc = chip8.pc as usize;
    let (code0, code1) = get_codes(chip8.memory, pc);
    let first_nib = code0 >> 4;

    print!("{:x} {:x} {:x} ", pc, code0, code1);

    match first_nib {
        0x00 => {
            match code1 {
                0xe0 => {
                    print!("{:-10}", "CLS");
                    chip8.screen.fill(0);
                    canvas.clear();
                    texture.update(None, &chip8.screen, 64 * 3).unwrap();
                    canvas.copy(&texture, None, None).unwrap();
                    canvas.present();
                },
                0xee => {
                    print!("{:-10}", "RTS");
                    chip8.pc = chip8.stack.pop().expect("chip8.stack should not be empty");
                    chip8.sp -= 1;
                    // chip8.pc += 2;
                    // disassemble(chip8, canvas, texture);
                },
                _ => print!("Unknown 0"),
            }
        },
        0x01 => {
            print!("{:-10} ${:01x}{:02x}", "JUMP", code0 & 0xf, code1);
            chip8.pc = ((code0 & 0xf) as u16) << 8 | code1 as u16;
            chip8.pc -= 2;
            // disassemble(chip8, canvas, texture, event_pump);
        },
        0x02 => {
            print!("{:-10} ${:01x}{:02x}", "CALL", code0 & 0xf, code1);
            chip8.sp += 1;
            chip8.stack.push(chip8.pc);
            let addr = ((code0 & 0xf) as u16) << 8 | code1 as u16;
            chip8.pc = addr;
            chip8.pc -= 2;
            // disassemble(chip8, canvas, texture, event_pump);
        },
        0x03 => {
            print!("{:-10} V{:01x},#${:02x}", "SKIP.EQ", code0 & 0xf, code1);

            if chip8.v[(code0 & 0xf) as usize] == code1 {
                chip8.pc += 2;
            }
        },
        0x04 => {
            print!("{:-10} V{:01x},#${:02x}", "SKIP.NE", code0 & 0xf, code1);
            if chip8.v[(code0 & 0xf) as usize] != code1 {
                chip8.pc += 2;
            }
        },
        0x05 => {
            print!("{:-10} V{:01x},V{:01x}", "SKIP.EQ", code0 & 0xf, code1 >> 4);
            if chip8.v[(code0 & 0xf) as usize] == chip8.v[(code1 >> 4) as usize] {
                chip8.pc += 2;
            }
        },
        0x06 => {
            print!("{:-10} V{:01x},#${:02x}", "MVI", code0 & 0xf, code1);
            chip8.v[(code0 & 0xf) as usize] = code1;
        },
        0x07 => {
            print!("{:-10} V{:01x},#{:02x}", "ADI", code0 & 0xf, code1);
            // chip8.v[(code0 & 0x0f) as usize] = chip8.v[(code0 & 0x0f) as usize] + code1;
            chip8.v[(code0 & 0x0f) as usize] = chip8.v[(code0 & 0x0f) as usize].overflowing_add(code1).0;
        },
        0x08 => {
            let last_nib: u8 = code1 & 0xf;
            match last_nib {
                0x0 => {
                    print!("{:-10} V{:01x},V{:01x}", "MOV.", code0 & 0xf, code1 & 0xf);
                    chip8.v[(code0 & 0xf) as usize] = chip8.v[(code1 >> 4) as usize];
                },
                0x1 => {
                    print!("{:-10} V{:01x},V{:01x}", "OR.", code0 & 0xf, code1 & 0x0f);
                    chip8.v[(code0 & 0xf) as usize] = chip8.v[(code0 & 0xf) as usize] | chip8.v[(code1 >> 4) as usize]; 
                    chip8.v[0xf] = 0;
                },
                0x2 => {
                    print!("{:-10} V{:01x},V{:01x}", "AND.", code0 & 0xf, code1 & 0x0f);
                    chip8.v[(code0 & 0xf) as usize] = chip8.v[(code0 & 0xf) as usize] & chip8.v[(code1 >> 4) as usize]; 
                    chip8.v[0xf] = 0;
                },
                0x3 => {
                    print!("{:-10} V{:01x},V{:01x}", "XOR.", code0 & 0xf, code1 & 0x0f);
                    chip8.v[(code0 & 0xf) as usize] = chip8.v[(code0 & 0xf) as usize] ^ chip8.v[(code1 >> 4) as usize]; 
                    chip8.v[0xf] = 0;
                },
                0x4 => {
                    print!("{:-10} V{:01x},V{:01x}", "ADD.", code0 & 0xf, code1 & 0x0f);


                    if chip8.v[(code0 & 0x0f) as usize].overflowing_add(chip8.v[(code1 >> 4) as usize]).1 {
                        chip8.v[(code0 & 0xf) as usize] = chip8.v[(code0 & 0xf) as usize].overflowing_add(chip8.v[(code1 >> 4) as usize]).0;
                        chip8.v[0xf] = 1;                        
                    } else {
                        chip8.v[(code0 & 0xf) as usize] = chip8.v[(code0 & 0xf) as usize].overflowing_add(chip8.v[(code1 >> 4) as usize]).0;
                        chip8.v[0xf] = 0;
                    }
                    
                    // chip8.v[(code0 & 0xf) as usize] = chip8.v[(code0 & 0xf) as usize].overflowing_add(chip8.v[(code1 >> 4) as usize]).0;
                },
                0x5 => {
                    print!("{:-10} V{:01x},V{:01x}", "SUB.", code0 & 0xf, code1 & 0x0f);

                    if chip8.v[(code0 & 0xf) as usize] > chip8.v[(code1 >> 4) as usize] {
                        chip8.v[(code0 & 0xf) as usize] = chip8.v[(code0 & 0xf) as usize].overflowing_sub(chip8.v[(code1 >> 4) as usize]).0;
                        chip8.v[0xf] = 1;
                    } else {
                        chip8.v[(code0 & 0xf) as usize] = chip8.v[(code0 & 0xf) as usize].overflowing_sub(chip8.v[(code1 >> 4) as usize]).0;
                        chip8.v[0xf] = 0;
                    }
                    
                    // chip8.v[(code0 & 0xf) as usize] = chip8.v[(code0 & 0xf) as usize] - chip8.v[(code1 >> 4) as usize];
                },
                0x6 => {
                    print!("{:-10} V{:01x},V{:01x}", "SHR.", code0 & 0xf, code1 & 0x0f);
                    chip8.v[(code0 & 0xf) as usize] = chip8.v[(code1 >> 4) as usize];

                    if (chip8.v[(code0 & 0xf) as usize] & 0x1) == 1 {
                        chip8.v[(code0 & 0xf) as usize] = chip8.v[(code0 & 0xf) as usize] >> 1; 
                        chip8.v[0xf] = 1;
                    } else {
                        chip8.v[(code0 & 0xf) as usize] = chip8.v[(code0 & 0xf) as usize] >> 1; 
                        chip8.v[0xf] = 0;
                    }
                    
                },
                0x7 => {
                    print!("{:-10} V{:01x},V{:01x}", "SUBN.", code0 & 0xf, code1 & 0x0f);
                    
                    chip8.v[(code0 & 0xf) as usize] = chip8.v[(code1 >> 4) as usize].overflowing_sub(chip8.v[(code0 & 0xf) as usize]).0;

                    if chip8.v[(code1 >> 4) as usize] > chip8.v[(code0 & 0xf) as usize] {
                        chip8.v[0xf] = 1;
                    } else {
                        chip8.v[0xf] = 0;
                    }
                    
                    // chip8.v[(code0 & 0xf) as usize] = chip8.v[(code1 >> 4) as usize] - chip8.v[(code0 & 0xf) as usize];
                },
                0xe => {
                    print!("{:-10} V{:01x},V{:01x}", "SHL.", code0 & 0xf, code1 & 0x0f);
                    
                    chip8.v[(code0 & 0xf) as usize] = chip8.v[(code1 >> 4) as usize];
                    
                    if ((chip8.v[(code0 & 0xf) as usize]) >> 7) == 1 {
                        chip8.v[(code0 & 0xf) as usize] <<= 1;
                        chip8.v[0xf] = 1;
                    } else {
                        chip8.v[(code0 & 0xf) as usize] <<= 1;
                        chip8.v[0xf] = 0;
                    }
                    
                },
                _ => print!("Unknown 8")
            }
        }
        0x09 => {
            print!("{:-10} V{:01x},V{:01x}", "SKIP.NE", code0 & 0xf, code1 >> 4);
            if chip8.v[(code0 & 0xf) as usize] != chip8.v[(code1 >> 4) as usize] {
                chip8.pc += 2;
            }
        },
        0x0a => {
            chip8.i = ((code0 & 0xf) as u16) << 8 | (code1) as u16; 
            let address_i: u8 = code0 & 0x0f;
            print!("{:-10} I,#${:01x}{:02x}", "MVI", address_i, code1);
        },
        0x0b => {
            print!("{:-10} I,#${:01x}{:02x}(V0)", "JUMP", code0 & 0xf, code1);
            chip8.pc = (((code0 & 0xf) as u16) << 8 | code1 as u16) + chip8.v[0] as u16;
            chip8.pc -= 2;
            // disassemble(chip8, canvas, texture, event_pump);
        },
        0x0c => {
            print!("{:-10} V{:01x}, #${:02x}", "RNDMSK", code0 & 0xf, code1);
            let random_byte: u8 = rand::thread_rng().gen_range(0..=255);
            chip8.v[(code0 & 0xf) as usize] = random_byte & code1;
        },
        0x0d => {
            print!("{:-10} V{:01x}, V{:01x}, #${}", "SPRITE", code0 & 0xf, code1 >> 4, code1 & 0xf);
            let addr = chip8.memory[chip8.i as usize];

            let width: u16 = 64;
            let height: u8 = 32;
            let mut v_x = chip8.v[(code0 & 0xf) as usize] % width as u8;
            let mut v_y = chip8.v[(code1 >> 4) as usize] % height;
            // let mut x = v_x;
            let num_of_bytes = code1 & 0xf;
            chip8.v[0xf] = 0;

            for b in chip8.i..chip8.i + num_of_bytes as u16 {
                let mut byte = chip8.memory[b as usize];
                // let mut byte = if chip8.i >= 0x200 {chip8.memory[b as usize]} else {chip8.font[b as usize]};
                let mut i: usize = 0;

                while i < 8 {
                    let pixel = byte >> 7;
                    byte = byte << 1;
                    i += 1;

                    let (r, g, b) = if pixel == 1 {sdl2::pixels::Color::WHITE.rgb()} 
                        else {sdl2::pixels::Color::BLACK.rgb()};
                    
                    let index = ((v_x * 3)) as usize + (v_y as usize * (width * 3) as usize);
            
                    if (chip8.screen[index] == 255) && (pixel == 1) {
                        chip8.v[0xf] = 1;
                    }

                    chip8.screen[index] ^= r;
                    chip8.screen[index + 1] ^= g;
                    chip8.screen[index + 2] ^= b;
                    
                    v_x = v_x + 1; 
                    
                    if v_x as u16 > width - 1 {
                        break;
                    }
                }

                    v_x = chip8.v[(code0 & 0xf) as usize] % width as u8;
                    // v_y = (v_y + 1) % height;
                    v_y += 1;
                    if v_y > (height - 1) {
                        break;
                    }
                }
                
                canvas.clear();
                texture.update(None, &chip8.screen, 64 * 3).unwrap();
                canvas.copy(&texture, None, None).unwrap();
                canvas.present();      
        },
        0x0e => {
            match code1 {
                0x9e => {
                    print!("{:-10} v{:01x}", "skipkey.y", code0 & 0xf);

                    let key_map = get_key_map();

                    let scancode_requested = key_map.iter()
                        .find_map(|(key, val)| if *val == (chip8.v[(code0 & 0xf) as usize]) {Some(key)} else {None});
                    
                    if keys.contains(scancode_requested.unwrap()) {
                        chip8.pc += 2;
                    }
                    
                },
                0xa1 => {
                    print!("{:-10} V{:01x}", "SKIPKEY.N", code0 & 0x0f);

                    let key_map = get_key_map();

                    let scancode_requested = key_map.iter()
                        .find_map(|(key, val)| if *val == (chip8.v[(code0 & 0xf) as usize]) {Some(key)} else {None});
                    
                    if !keys.contains(scancode_requested.unwrap()) {
                        chip8.pc += 2;
                    }

                },
                _ => print!("Unknown e")
            }
        },
        0x0f => {
            match code1 {
                0x07 => { 
                    print!("{:-10} V{:01x}, DELAY", "MOV", code0 & 0xf);
                    chip8.v[(code0 & 0xf) as usize] = chip8.delay;
                },
                0x0a => {
                    print!("{:-10} V{:01x}", "KEY", code0 & 0x0f);
                    let key_map = get_key_map();

                    let scancode_requested = key_map.iter()
                        .find_map(|(key, val)| if *val == (chip8.v[(code0 & 0xf) as usize]) {Some(key)} else {None});

                    'running: loop {

                            for event in event_pump.poll_iter() {
                                let key = match event {
                                    Event::KeyDown {scancode: Some(Scancode::Num1), ..} => {
                                        chip8.v[(code0 & 0xf) as usize] = *key_map.get(&Scancode::Num1).unwrap();
                                        break 'running;
                                    },
                                    Event::KeyDown {scancode: Some(Scancode::Num2), ..} => {
                                        chip8.v[(code0 & 0xf) as usize] = *key_map.get(&Scancode::Num2).unwrap();
                                        break 'running;
                                    } 
                                    Event::KeyDown {scancode: Some(Scancode::Num3), ..} => {
                                        chip8.v[(code0 & 0xf) as usize] = *key_map.get(&Scancode::Num3).unwrap();
                                        break 'running;
                                    },
                                    Event::KeyDown {scancode: Some(Scancode::Num4), ..} => {
                                        chip8.v[(code0 & 0xf) as usize] = *key_map.get(&Scancode::Num4).unwrap();
                                        break 'running;
                                    },
                                    Event::KeyDown {scancode: Some(Scancode::Q), ..} => {
                                        chip8.v[(code0 & 0xf) as usize] = *key_map.get(&Scancode::Q).unwrap();
                                        break 'running;
                                    },
                                    Event::KeyDown {scancode: Some(Scancode::W), ..} => {
                                        chip8.v[(code0 & 0xf) as usize] = *key_map.get(&Scancode::W).unwrap();
                                        break 'running; 
                                    },
                                    Event::KeyDown {scancode: Some(Scancode::E), ..} => {
                                        chip8.v[(code0 & 0xf) as usize] = *key_map.get(&Scancode::E).unwrap();
                                        break 'running;
                                    },
                                    Event::KeyDown {scancode: Some(Scancode::R), ..} => {
                                        chip8.v[(code0 & 0xf) as usize] = *key_map.get(&Scancode::R).unwrap();
                                        break 'running; 
                                    },
                                    Event::KeyDown {scancode: Some(Scancode::A), ..} => {
                                        chip8.v[(code0 & 0xf) as usize] = *key_map.get(&Scancode::A).unwrap();
                                        break 'running;
                                    },
                                    Event::KeyDown {scancode: Some(Scancode::S), ..} => {
                                        chip8.v[(code0 & 0xf) as usize] = *key_map.get(&Scancode::S).unwrap();
                                        break 'running;
                                    },
                                    Event::KeyDown {scancode: Some(Scancode::D), ..} => {
                                        chip8.v[(code0 & 0xf) as usize] = *key_map.get(&Scancode::D).unwrap();
                                        break 'running; 
                                    },
                                    Event::KeyDown {scancode: Some(Scancode::F), ..} => {
                                        chip8.v[(code0 & 0xf) as usize] = *key_map.get(&Scancode::F).unwrap();
                                        break 'running;
                                    },
                                    Event::KeyDown {scancode: Some(Scancode::Z), ..} => {
                                        chip8.v[(code0 & 0xf) as usize] = *key_map.get(&Scancode::Z).unwrap();
                                        break 'running;
                                    },
                                    Event::KeyDown {scancode: Some(Scancode::X), ..} => {
                                        chip8.v[(code0 & 0xf) as usize] = *key_map.get(&Scancode::X).unwrap();
                                        break 'running;
                                    },
                                    Event::KeyDown {scancode: Some(Scancode::C), ..} => {
                                        chip8.v[(code0 & 0xf) as usize] = *key_map.get(&Scancode::C).unwrap();
                                        break 'running;
                                    },
                                    Event::KeyDown {scancode: Some(Scancode::V), ..} => {
                                        chip8.v[(code0 & 0xf) as usize] = *key_map.get(&Scancode::V).unwrap();
                                        break 'running;
                                    },
                                    _ => continue, 
                                };
                            }
                            if chip8.delay > 0 {
                                chip8.delay -= 1;
                                println!("DELAY {}", &chip8.delay);
                            }
                        }
                }, 
                0x15 => {
                    print!("{:-10} DELAY,V{:01x}", "MOV", code0 & 0x0f);
                    chip8.delay = chip8.v[(code0 & 0xf) as usize];
                    println!("\n\nDELAY: {}", chip8.delay);
                },
                0x18 => print!(" SOUND, V MOV"),
                0x1e => {
                    print!("{:-10} I,V{:01x}", "ADI", code0 & 0x0f);
                    chip8.i = chip8.i + chip8.v[(code0 & 0xf) as usize] as u16;
                },
                0x29 => {
                    let v_x = chip8.v[(code0 & 0xf) as usize];
                    chip8.i = (v_x * 5) as u16;
                },
                0x33 => {
                    print!("{:-10} (I),V{:01x}", "MOVBCD", code0 & 0x0f);
                    let v_x = chip8.v[(code0 & 0xf) as usize];
                    let mut num = v_x; 
                    // store digits of decimal value of v_x in I, I + 1, I + 2
                    let ones = num % 10;
                    num /= 10; 
                    let tens = num % 10;
                    num /= 10;
                    let hundreds = num % 10;


                    chip8.memory[chip8.i as usize] = hundreds;
                    chip8.memory[(chip8.i + 1) as usize] = tens;
                    chip8.memory[(chip8.i + 2) as usize] = ones;
                },
                0x55 => {
                    print!("{:-10} I,V0-V{:01x}", "MOVM", code0 & 0x0f);
                    chip8.memory[chip8.i as usize ..= (chip8.i as usize + (code0 & 0xf) as usize)].copy_from_slice(
                        &chip8.v[0..=(code0 & 0xf) as usize]
                    );
                    chip8.i += (code0 & 0xf) as u16 + 1;
                },
                0x65 => {
                    print!("{:-10} V0-V{:01x},(I)", "MOVM", code0 & 0x0f);
                    chip8.v[0..=(code0 & 0xf) as usize].copy_from_slice(
                        &chip8.memory[chip8.i as usize ..= (chip8.i as usize + (code0 & 0xf) as usize)]
                    );
                    chip8.i += (code0 & 0xf) as u16 + 1;
                },
                _ => print!("Unknown f")
            }
        },
        _ => print!("wrong input"),
    }

}