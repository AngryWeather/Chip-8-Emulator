
use std::env;
use std::io;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::SeekFrom;
use std::io::Seek;

fn main() -> io::Result<()>{
    let args: Vec<String> = env::args().collect();
    let file_path = &args[1];

    let f = File::open(file_path)?;
    let mut reader = BufReader::new(f);
    let mut buffer: Vec<u8> = Vec::new();
    
    let f_size = reader.seek(SeekFrom::End(0))?;
    reader.rewind()?;
    reader.read_to_end(&mut buffer)?;
    
    // Chip-8 puts programs in memory at 0x200
    let mut pc: usize = 0x200;

    // Read.
    while (pc as u64) < f_size {
        disassemble(&buffer, pc);
        pc += 2;
        print!("\n");
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
    screen: u8,
}

impl Chip8State {
    fn new(memory: [u8; 1024 * 4], v: [u8;16], i: u16, delay: u8, sound: u8) -> Self {Chip8State {
        memory,
        screen: memory[0xf00],
        sp: 0xfa0,
        pc: 0x200,
        v,
        i,
        delay,
        sound,
        }
    }
}

fn disassemble(code_buffer: &Vec<u8>, pc: usize) {
    let code0 = &code_buffer[pc];
    let code1 = &code_buffer[pc + 1];
    let first_nib = code0 >> 4;

    print!("{:x} {:x} {:x} ", pc, code0, code1);

    match first_nib {
        0x00 => {
            match code1 {
                0xe0 => print!("{:-10}", "CLS"),
                0xee => print!("{:-10}", "RTS"),
                _ => print!("Unknown 0"),
            }
        },
        0x01 => print!("{:-10} ${:01x}{:02x}", "JUMP", code0 & 0xf, code1),
        0x02 => print!("{:-10} ${:01x}{:02x}", "CALL", code0 & 0xf, code1),
        0x03 => print!("{:-10} V{:01x},#${:02x}", "SKIP.EQ", code0 & 0xf, code1),
        0x04 => print!("{:-10} V{:01x},#${:02x}", "SKIP.NE", code0 & 0xf, code1),
        0x05 => print!("{:-10} V{:01x},V{:01x}", "SKIP.EQ", code0 & 0xf, code1 >> 4),
        0x06 => print!("{:-10} V{:01x},#${:02x}", "MVI", code0 & 0xf, code1),
        0x07 => print!("{:-10} V{:01x},#{:02x}", "ADI", code0 & 0xf, code1),
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
            let address_i: u8 = code0 & 0x0f;
            print!("{:-10} I,#${:01x}{:02x}", "MVI", address_i, code1);
        },
        0x0b => print!("{:-10} I,#${:01x}{:02x}(V0)", "JUMP", code0 & 0xf, code1),
        0x0c => print!("{:-10} V{:01x}, #${:02x}", "RNDMSK", code0 & 0xf, code1),
        0x0d => print!("{:-10} V{:01x}, V{:01x}, #${:01x}", "SPRITE", code0 & 0xf, code1 >> 4, code1 & 0xf),
        0x0e => {
            match code1 {
                0x9e => print!("{:-10} V{:01x}", "SKIPKEY.Y", code0 & 0xf),
                0xa1 => print!("{:-10} V{:01x}", "SKIPKEY.N", code0 & 0x0f),
                _ => print!("Unknown e")
            }
        },
        0x0f => {
            match code1 {
                0x07 => print!("{:-10} V{:01x}, DELAY", "MOV", code0 & 0xf),
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