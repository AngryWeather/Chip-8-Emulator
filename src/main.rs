extern crate sdl2;
use sdl2::event::Event;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Canvas;
use sdl2::render::Texture;
use sdl2::video::Window;
use std::env;
use std::io;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::SeekFrom;
use std::io::Seek;

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
        
        // chip8-8 puts programs in memory at 0x200
        let mut chip8 = Chip8State::new([0;1024*4], [0; 64 * 32 * 3], 
            [0; 16], 0x0, 0x0, 0x0);    

        chip8.memory[0x200 .. (0x200 + &buffer.len())].copy_from_slice(&buffer[..]);

    'running: loop {
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit{..} => break 'running,
                _ => {},
            }
        }
        while (chip8.pc) < 0x200 + buffer.len() as u16{
            println!("pc out: {:x}", &chip8.pc);
            disassemble(&mut chip8, &mut canvas, &mut texture);
            chip8.pc += 2;
            print!("\n"); 
            ::std::thread::sleep(std::time::Duration::new(0, 100000000));
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

fn disassemble(chip8: &mut Chip8State, canvas: &mut Canvas<Window>, texture: &mut Texture) {
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
                    texture.update(None, &chip8.screen, 64 * 4).unwrap();
                    canvas.copy(&texture, None, None).unwrap();
                    canvas.present();
                },
                0xee => {
                    print!("{:-10}", "RTS");
                    chip8.pc = chip8.stack.pop().expect("chip8.stack should not be empty") + 2;
                    chip8.sp -= 1;
                    println!();
                    disassemble(chip8, canvas, texture);
                },
                _ => print!("Unknown 0"),
            }
        },
        0x01 => {
            print!("{:-10} ${:01x}{:02x}", "JUMP", code0 & 0xf, code1);
            chip8.pc = ((code0 & 0xf) as u16) << 8 | code1 as u16;
            println!("pc: {:x}", &chip8.pc);
            disassemble(chip8, canvas, texture);
        },
        0x02 => {
            print!("{:-10} ${:01x}{:02x}", "CALL", code0 & 0xf, code1);
            chip8.sp += 1;
            chip8.stack.push(chip8.pc);
            let addr = ((code0 & 0xf) as u16) << 8 | code1 as u16;
            chip8.pc = addr;
            disassemble(chip8, canvas, texture);
        },
        0x03 => {
            print!("{:-10} V{:01x},#${:02x}", "SKIP.EQ", code0 & 0xf, code1);

            if chip8.v[code0 as usize & 0xf as usize] == code1 {
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
            println!("\n{:0x?}", chip8.v);
        },
        0x07 => {
            print!("{:-10} V{:01x},#{:02x}", "ADI", code0 & 0xf, code1);
            // chip8.v[(code0 & 0x0f) as usize] = chip8.v[(code0 & 0x0f) as usize] + code1;
            chip8.v[(code0 & 0x0f) as usize] = chip8.v[(code0 & 0x0f) as usize].overflowing_add(code1).0;
            println!("{:x?}", chip8.v);
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
                },
                0x2 => {
                    print!("{:-10} V{:01x},V{:01x}", "AND.", code0 & 0xf, code1 & 0x0f);
                    chip8.v[(code0 & 0xf) as usize] = chip8.v[(code0 & 0xf) as usize] & chip8.v[(code1 >> 4) as usize]; 
                },
                0x3 => {
                    print!("{:-10} V{:01x},V{:01x}", "XOR.", code0 & 0xf, code1 & 0x0f);
                    chip8.v[(code0 & 0xf) as usize] = chip8.v[(code0 & 0xf) as usize] ^ chip8.v[(code1 >> 4) as usize]; 
                },
                0x4 => {
                    print!("{:-10} V{:01x},V{:01x}", "ADD.", code0 & 0xf, code1 & 0x0f);
                    if chip8.v[(code0 & 0x0f) as usize].overflowing_add(code1 >> 4).1 {
                        chip8.v[0xf] = 1;                        
                    } else {
                        chip8.v[0xf] = 0;
                    }
                    
                    chip8.v[(code0 & 0xf) as usize] = chip8.v[(code0 & 0xf) as usize].overflowing_add(chip8.v[(code1 >> 4) as usize]).0;
                },
                0x5 => {
                    print!("{:-10} V{:01x},V{:01x}", "SUB.", code0 & 0xf, code1 & 0x0f);
                    if chip8.v[(code0 & 0xf) as usize] >  chip8.v[(code1 >> 4) as usize] {
                        chip8.v[0xf] = 1;
                    } else {
                        chip8.v[0xf] = 0;
                    }
                    
                   chip8.v[(code0 & 0xf) as usize] = chip8.v[(code0 & 0xf) as usize].overflowing_sub(chip8.v[(code1 >> 4) as usize]).0;
                },
                0x6 => {
                    print!("{:-10} V{:01x},V{:01x}", "SHR.", code0 & 0xf, code1 & 0x0f);
                    if (chip8.v[(code0 & 0xf) as usize] & 0x1) == 1 {
                        chip8.v[0xf] = 1;
                    } else {
                        chip8.v[0xf] = 0;
                    }
                    
                    chip8.v[(code0 & 0xf) as usize] = chip8.v[(code0 & 0xf) as usize] >> 1; 
                },
                0x7 => {
                    print!("{:-10} V{:01x},V{:01x}", "SUBN.", code0 & 0xf, code1 & 0x0f);
                    
                    if chip8.v[(code1 >> 4) as usize] > chip8.v[(code0 & 0xf) as usize] {
                        chip8.v[0xf] = 1;
                    } else {
                        chip8.v[0xf] = 0;
                    }
                    
                    chip8.v[(code0 & 0xf) as usize] = chip8.v[(code1 >> 4) as usize] - chip8.v[(code0 & 0xf) as usize];
                },
                0xe => {
                    print!("{:-10} V{:01x},V{:01x}", "SHL.", code0 & 0xf, code1 & 0x0f);
                    
                    if ((chip8.v[(code0 & 0xf) as usize] & 0x8) >> 7) == 1 {
                        chip8.v[0xf] = 1;
                    } else {
                        chip8.v[0xf] = 0;
                    }
                    
                    chip8.v[(code0 & 0xf) as usize] = chip8.v[(code0 & 0xf) as usize] << 1;
                },
                _ => print!("Unknown 8")
            }
        }
        0x09 => {
            print!("{:-10} V{:01x},V{:01x}", "SKIP.NE", code0 & 0xf, code1 & 0x0f);
            if chip8.v[(code0 & 0xf) as usize] != chip8.v[(code1 >> 4) as usize] {
                chip8.pc += 2;
            }
        },
        0x0a => {
            chip8.i = ((code0 & 0xf) as u16) << 8 | (code1) as u16; 
            let address_i: u8 = code0 & 0x0f;
            print!("{:-10} I,#${:01x}{:02x}", "MVI", address_i, code1);
        },
        0x0b => print!("{:-10} I,#${:01x}{:02x}(V0)", "JUMP", code0 & 0xf, code1),
        0x0c => print!("{:-10} V{:01x}, #${:02x}", "RNDMSK", code0 & 0xf, code1),
        0x0d => {
            print!("{:-10} V{:01x}, V{:01x}, #${}", "SPRITE", code0 & 0xf, code1 >> 4, code1 & 0xf);
            let addr = chip8.memory[chip8.i as usize];

            let width: u8 = 64;
            let height: u8 = 32;
            let mut v_x = chip8.v[(code0 & 0xf) as usize] % 64;
            let mut v_y = chip8.v[(code1 >> 4) as usize] % 32;
            let num_of_bytes = code1 & 0xf;

            for x in chip8.i..chip8.i + num_of_bytes as u16 {
                let mut byte = chip8.memory[x as usize];
                let mut i: usize = 0;

                while i < 8 {
                    let pixel = (byte & 0x80) >> 7;
                    byte = byte << 1;
                    i += 1;

                    let (r, g, b) = if pixel == 1 {sdl2::pixels::Color::WHITE.rgb()} 
                        else {sdl2::pixels::Color::BLACK.rgb()};
                    
                    let index = (v_y as usize * (width * 3) as usize) as usize + ( v_x * 3) as usize;
                    chip8.screen[index] ^= r;
                    chip8.screen[index + 1] ^= g;
                    chip8.screen[index + 2] ^= b;
                    v_x += 1;
                }

                    v_x = chip8.v[(code0 & 0xf) as usize] % 64;
                    v_y += 1;
                }
                
                canvas.clear();
                texture.update(None, &chip8.screen, 64 * 3).unwrap();
                canvas.copy(&texture, None, None).unwrap();
                canvas.present();      
        },
        0x0e => {
            match code1 {
                0x9e => print!("{:-10} V{:01x}", "SKIPKEY.Y", code0 & 0xf),
                0xa1 => print!("{:-10} V{:01x}", "SKIPKEY.N", code0 & 0x0f),
                _ => print!("Unknown e")
            }
        },
        0x0f => {
            match code1 {
                0x07 => { 
                    print!("{:-10} V{:01x}, DELAY", "MOV", code0 & 0xf);
                },
                0x0a => print!("{:-10} V{:01x}", "KEY", code0 & 0x0f),
                0x15 => print!("{:-10} DELAY,V{:01x}", "MOV", code0 & 0x0f),
                0x18 => print!("{:-10} SOUND, V{:01x}", "MOV", code0 * 0x0f),
                0x1e => print!("{:-10} I,V{:01x}", "ADI", code0 & 0x0f),
                0x29 => print!("{:-10} I,V{:01x}", "SPRITECHAR", code0 & 0x0f),
                0x33 => {
                    print!("{:-10} (I),V{:01x}", "MOVBCD", code0 & 0x0f);
                    let v_x = chip8.v[(code0 & 0xf) as usize];
                    let mut num = v_x; 
                    println!("num: {num}");
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
                },
                0x65 => {
                    print!("{:-10} V0-V{:01x},(I)", "MOVM", code0 & 0x0f);
                    chip8.v[0..=(code0 & 0xf) as usize].copy_from_slice(
                        &chip8.memory[chip8.i as usize ..= (chip8.i as usize + (code0 & 0xf) as usize)]
                    )
                },
                _ => print!("Unknown f")
            }
        },
        _ => print!("wrong input"),
    }

}