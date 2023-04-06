extern crate sdl2;
use sdl2::event::Event;
use sdl2::libc::SCTP_STREAM_RESET_INCOMING;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormat;
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

    let window = video_subsystem.window("chip8-8 emulator",640, 320)
        .position_centered()
        .build()
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut canvas = window.into_canvas().build().unwrap();


        let creator = canvas.texture_creator();
        let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 64, 32).unwrap();
        canvas.set_scale(10.0, 10.0).unwrap();


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
        // texture.update(None, &chip8.screen, 8);
// // 
    'running: loop {
        // canvas.set_draw_color(Color::RGB(1, 1, 1));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit{..} => break 'running,
                _ => {},
            }
        }
        canvas.present();
        while (chip8.pc) < 0x200 + buffer.len() as u16{
            disassemble(&mut chip8, &mut canvas, &mut texture);
            chip8.pc += 2;
            print!("\n"); 
            ::std::thread::sleep(std::time::Duration::new(0, 1000000000));
        }
    }

    // Read.
    // while (chip8.pc) < 0x200 + buffer.len() as u16{
    //     disassemble(&mut chip8, &mut canvas, &mut texture);
    //     chip8.pc += 2;
    //     print!("\n");
        
    // }
   
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
}

impl Chip8State {
    fn new(memory: [u8; 1024 * 4], screen:  [u8; 64 * 32 * 3], 
        v: [u8;16], i: u16, delay: u8, sound: u8) -> Self {Chip8State {
            memory,
            screen,
            sp: 0x0,
            pc: 0x200,
            v,
            i: 0x0,
            delay,
            sound,
        }
    }
}

fn disassemble(chip8: &mut Chip8State, canvas: &mut Canvas<Window>, texture: &mut Texture) {
    let pc = chip8.pc as usize;
    let code0 = &chip8.memory[pc];
    let code1 = &chip8.memory[pc + 1];
    let first_nib = code0 >> 4;

    print!("{:x} {:x} {:x} ", pc, code0, code1);

    match first_nib {
        0x00 => {
            match code1 {
                0xe0 => {
                    print!("{:-10}", "CLS");
                    chip8.screen.fill(0);
                    // println!("{:?}", chip8.screen);
                    canvas.clear();
                    // println!("texture: {:?}", canvas.read_pixels(None, PixelFormatEnum::RGB888));
                    // println!("{:?}", canvas.output_size());
                    // canvas.read_pixels(None, PixelFormatEnum::RGB888);
                    texture.update(None, &chip8.screen, 64).unwrap();
                    canvas.copy(texture, None, None).unwrap();
                    canvas.present();
                },
                0xee => print!("{:-10}", "RTS"),
                _ => print!("Unknown 0"),
            }
        },
        0x01 => {
            print!("{:-10} ${:01x}{:02x}", "JUMP", code0 & 0xf, code1);
            chip8.pc = (*code0 as u16) << 8 | *code1 as u16;
        },
        0x02 => print!("{:-10} ${:01x}{:02x}", "CALL", code0 & 0xf, code1),
        0x03 => print!("{:-10} V{:01x},#${:02x}", "SKIP.EQ", code0 & 0xf, code1),
        0x04 => print!("{:-10} V{:01x},#${:02x}", "SKIP.NE", code0 & 0xf, code1),
        0x05 => print!("{:-10} V{:01x},V{:01x}", "SKIP.EQ", code0 & 0xf, code1 >> 4),
        0x06 => {
            print!("{:-10} V{:01x},#${:02x}", "MVI", code0 & 0xf, code1);
            chip8.v[(code0 & 0xf) as usize] = *code1;
            println!("\n{:0x?}", chip8.v);
        },
        0x07 => {
            print!("{:-10} V{:01x},#{:02x}", "ADI", code0 & 0xf, code1);
            chip8.v[(code0 & 0x0f) as usize] = chip8.v[(code0 & 0x0f) as usize] + code1;
            println!("{:x?}", chip8.v);
        },
        0x08 => {
            let last_nib: u8 = code1 & 0xf;
            match last_nib {
                0x0 => print!("{:-10} V{:01x},V{:01x}", "MOV.", code0 & 0xf, code1 & 0xf),
                0x1 => print!("{:-10} V{:01x},V{:01x}", "OR.", code0 & 0xf, code1 & 0x0f),
                0x2 => print!("{:-10} V{:01x},V{:01x}", "AND.", code0 & 0xf, code1 & 0x0f),
                0x3 => print!("{:-10} V{:01x},V{:01x}", "XOR.", code0 & 0xf, code1 & 0x0f),
                0x4 => print!("{:-10} V{:01x},V{:01x}", "ADD.", code0 & 0xf, code1 & 0x0f),
                0x5 => print!("{:-10} V{:01x},V{:01x}", "SUB.", code0 & 0xf, code1 & 0x0f),
                0x6 => print!("{:-10} V{:01x},V{:01x}", "SHR.", code0 & 0xf, code1 & 0x0f),
                0x7 => print!("{:-10} V{:01x},V{:01x}", "SUBN.", code0 & 0xf, code1 & 0x0f),
                0xe => print!("{:-10} V{:01x},V{:01x}", "SHL.", code0 & 0xf, code1 & 0x0f),
                _ => print!("Unknown 8")
            }
        }
        0x09 => print!("{:-10} V{:01x},V{:01x}", "SKIP.NE", code0 & 0xf, code1 & 0x0f),
        0x0a => {
            chip8.i = ((code0 & 0xf) as u16) << 8 | (*code1) as u16; 
            println!("chip8.i: {:x}", chip8.i);
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

            let color: u8;

            for x in chip8.i..chip8.i + num_of_bytes as u16 {
                let mut byte = chip8.memory[x as usize];
                let mut i: usize = 0;

                // println!("x: {:x?}", x);

                while i < 8 {
                    let pixel = (byte & 0x80) >> 7;
                    byte = byte << 1;
                    i += 1;
                    let r = (v_x as u16 + (width as u16 * v_y as u16) as u16) as usize;
                    let g = ((v_x) as u16 + (width as u16 * v_y as u16) as u16) as usize;
                    let b = ((v_x) as u16 + (width as u16 * v_y as u16) as u16) as usize;
                    let color: u8 = if pixel == 1 {255} else {0};
                    
                    chip8.screen[r] = chip8.screen[r] ^ color;
                    chip8.screen[r + 1] = chip8.screen[r + 1] ^ color;
                    chip8.screen[r+2] = chip8.screen[r+2] ^ color;
                    // chip8.screen[(v_x as u16 + (width as u16 * v_y as u16) as u16) as usize] = (chip8.screen[(v_x as u16 + 
                    //     (width as u16 * v_y as u16) as u16) as usize] ^ color);


                    // println!("v_x: {v_x}");
                    // println!("v_y: {v_y}");
                    v_x += 3;

                        }
                    
                        v_x = chip8.v[(code0 & 0xf) as usize];
                        v_y += 1;
                }

                // println!("\naddr: {:x}", addr);
                canvas.clear();
                texture.update(None, &chip8.screen, 64).unwrap();
                canvas.copy(texture, None, None).unwrap();
                canvas.present();      

                println!("canvas size: {:?}", texture.query())
                    // println!("\n{:?}", chip8.screen);

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
                0x33 => print!("{:-10} (I),V{:01x}", "MOVBCD", code0 & 0x0f),
                0x55 => print!("{:-10} I,V0-V{:01x}", "MOVM", code0 & 0x0f),
                0x65 => print!("{:-10} V0-V{:01x},(I)", "MOVM", code0 & 0x0f),
                _ => print!("Unknown f")
            }
        },
        _ => print!("wrong input"),
    }

}