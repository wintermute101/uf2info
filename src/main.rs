use anyhow::{Result,Error, anyhow};
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use clap::{Parser};
use uftwo::Block;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    inputfile: PathBuf,

    #[arg(short, long)]
    verbose: bool,

    outputfile: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut input_file = File::open(cli.inputfile.to_owned())?;
    let mut binary: Vec<u8> = vec![];

    let mut total_blocks = 0;

    let mut read_total_blocks = 0;

    let output = if let Some(output) = cli.outputfile {
        Some(File::create(output)?)
    } else {
        None
    };

    loop {
        let mut buf = [0; 512];

        let bytes = input_file.read(&mut buf)?;

        if bytes == 0 {
            break;
        }
        else if bytes < buf.len() {
            return Err(anyhow!("Read {} bytes, it is not block size", bytes));
        }

        let block = Block::from_bytes(&buf).map_err(Error::msg)?;

        if cli.verbose{
            println!("Block {} flags {:?} target_addr 0x{:X}-0x{:X} data_len {} total_blocks {}", total_blocks, block.flags, block.target_addr, block.target_addr+block.data_len, block.data_len, block.total_blocks);
        }

        read_total_blocks = block.total_blocks;

        if block.block != total_blocks{
            eprintln!("Block no {} does not match read blocks no {}", block.block, total_blocks);
        }

        binary.extend(&buf[0..(block.data_len as usize)]);

        total_blocks += 1;
    }

    if read_total_blocks != total_blocks{
        eprintln!("Read total blocks {} does not match total blocks in block {}", total_blocks, read_total_blocks);
    }

    if let Some(mut output) = output {
        output.write_all(&binary)?;
        output.flush()?;
    }

    let filename = cli.inputfile.to_string_lossy();

    println!("File {} len {} total blocks {total_blocks} binary len {}", filename, total_blocks*512, binary.len());

    Ok(())
}
