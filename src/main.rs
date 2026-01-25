use anyhow::{Result,Error, anyhow};
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use clap::{Parser};
use uftwo::Block;
use std::fmt::Display;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    inputfile: PathBuf,

    #[arg(short, long)]
    verbose: bool,

    outputfile: Option<PathBuf>,
}

struct MemoryRegions{
    regions: Vec<Vec<u32>>
}

impl MemoryRegions {
    fn new() -> Self{
        MemoryRegions { regions: vec![] }
    }

    fn add_region(&mut self, new_start: u32, new_end: u32){
        match match (
            self.regions.binary_search_by(|v| v[1].cmp(&new_start)),
            self.regions.binary_search_by(|v| v[0].cmp(&new_end)),
        ) {
            (Err(start), Err(end)) | (Ok(start), Err(end)) => (start, end),
            (Err(start), Ok(end)) | (Ok(start), Ok(end)) => (start, end + 1),
        } {
            (start, end) => match match start != end {
                true => vec![
                    new_start.min(self.regions[start][0]),
                    new_end.max(self.regions[end - 1][1]),
                ],
                false => vec![new_start, new_end],
            } {
                r => {
                    self.regions.splice(start..end, vec![r]);
                },
            },
        }
    }
}

impl Display for MemoryRegions{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in self.regions.iter(){
            f.write_fmt(format_args!("0x{:X}-0x{:X} ", i[0], i[1]))?;
        }
        Ok(())
    }
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

    let mut regions = MemoryRegions::new();

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

        if block.data_len > 0{
            regions.add_region(block.target_addr, block.target_addr + block.data_len);
        }

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

    println!("File {}\nInput file size {} bytes\nTotal blocks {total_blocks}\nBinary len {} bytes\nMemory regions {}", filename, total_blocks*512, binary.len(), regions);

    Ok(())
}
