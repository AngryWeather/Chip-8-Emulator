
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

fn disassemble(code_buffer: &Vec<u8>, pc: usize) {
    let code0 = &code_buffer[pc];
    let code1 = &code_buffer[pc + 1];
    let first_nib = code0 >> 4;

    print!("{:x} {:x} {:x} ", pc, code0, code1);

    match first_nib {
        0x00 => print!("0 not handled yet"),
        0x01 => print!("1 not handled yet"),
        0x02 => print!("2 not handled yet"),
        0x03 => print!("3 not handled yet"),
        0x04 => print!("4 not handled yet"),
        0x05 => print!("5 not handled yet"),
        0x06 => {
            let reg: u8 = code0 & 0x0f;
            print!("{:-10} V{:01x},#${:02x}", "MVI", reg, code1);
        },
        0x07 => print!("7 not handled yet"),
        0x08 => print!("8 not handled yet"),
        0x09 => print!("9 not handled yet"),
        0x0a => {
            let address_i: u8 = code0 & 0x0f;
        },
        0x0b => print!("b not handled yet"),
        0x0c => print!("c not handled yet"),
        0x0d => print!("d not handled yet"),
        0x0e => print!("e not handled yet"),
        0x0f => print!("f not handled yet"),
        _ => print!("wrong input"),
    }

}
