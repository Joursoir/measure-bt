#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;
use alloc::format;
use uefi::prelude::*;
use uefi::runtime::*;
use uefi::runtime::{VariableVendor};
use uefi::{Guid, CStr16};
use uefi::println;

use uefi::CString16;
use uefi::fs::{FileSystem, FileSystemResult};
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::boot::{self, ScopedProtocol};

const FILENAME_STATS: &str = "measure_count.txt";
const AMOUNT_OF_RESTARTS: u8 = 100;

fn read_file(path: &str) -> FileSystemResult<String> {
    let path: CString16 = CString16::try_from(path).unwrap();
    let fs: ScopedProtocol<SimpleFileSystem> = boot::get_image_file_system(boot::image_handle()).unwrap();
    let mut fs = FileSystem::new(fs);
    fs.read_to_string(path.as_ref())
}

fn write_file(path: &str, buffer: &[u8]) -> FileSystemResult<()> {
    let path: CString16 = CString16::try_from(path).unwrap();
    let fs: ScopedProtocol<SimpleFileSystem> = boot::get_image_file_system(boot::image_handle()).unwrap();
    let mut fs = FileSystem::new(fs);
    fs.write(path.as_ref(), buffer)
}

// fn get_count() -> Result {

// }

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap(); // Initialize uefi::helpers

    let mut buffer = [0u8; 1];
    let mut var_name_str = [0u16; 16];
    let var_name = CStr16::from_str_with_buf("MeasureCount", &mut var_name_str).unwrap();

    let vendor = VariableVendor(Guid::from_bytes([12, 34, 56, 78, 12, 34, 12, 34, 12, 34, 12, 34, 56, 78, 90, 12]));
    let attributes = VariableAttributes::NON_VOLATILE | VariableAttributes::BOOTSERVICE_ACCESS;
    let mut measure_count = AMOUNT_OF_RESTARTS;

    match uefi::runtime::get_variable(var_name, &vendor, &mut buffer) {
        Ok((variable, attrs)) => {
            println!("{}\n  {:?}\n  Content: {:?}", var_name, attrs, variable);

            if attributes != attrs {
                println!("Error!");
            }

            measure_count = variable[0];
            if measure_count > AMOUNT_OF_RESTARTS {
                measure_count = AMOUNT_OF_RESTARTS;
            }

            if measure_count == 0 {
                println!("We are done!");

                if let Err(err) = set_variable(var_name, &vendor, attributes, &[]) {
                    println!("Error setting variable: {}", err);
                    return err.status();
                }

                return Status::SUCCESS
            }
        }
        Err(err) => {
            if err.status() != Status::NOT_FOUND {
                println!("Error getting variable: {}", err);
                // exit
            }

            // FIRST TIME

            if let Err(err) = write_file(FILENAME_STATS, &[]) {
                println!("Error writing file: {}", err);
                return Status::NOT_FOUND;
            }

            if let Err(err) = set_variable(var_name, &vendor, attributes, &buffer) {
                println!("Error setting variable: {}", err);
                return err.status();
            }
        }
    }

    measure_count -= 1;
    buffer[0] = measure_count;
    if let Err(err) = set_variable(var_name, &vendor, attributes, &buffer) {
        println!("Error setting variable: {}", err);
        return err.status();
    }

    let content = match read_file(FILENAME_STATS) {
        Ok(text) => text,
        Err(err) => {
            println!("Error reading file: {}", err);
            return Status::NOT_FOUND;
        }
    };

    let time = uefi::runtime::get_time().unwrap();
    let sec: u64 = (time.hour() as u64) * 3600 + (time.minute() as u64) * 60 + (time.second() as u64);
    let text = format!("{content}{}: {} seconds\n", buffer[0], sec);

    // println!("Readed from file: {content}");
    // println!("Writing to file: {text}");

    let txt_bytes = text.as_bytes();
    if let Err(err) = write_file(FILENAME_STATS, txt_bytes) {
        println!("Error writing file: {}", err);
        return Status::NOT_FOUND;
    }

    reset(ResetType::COLD, Status::SUCCESS, None);
    // Status::SUCCESS
}