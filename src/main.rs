#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use uefi::prelude::*;
use uefi::runtime::*;
use uefi::runtime::{VariableVendor};
use uefi::{guid, CStr16};
use uefi::println;

use uefi::CString16;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::proto::media::file::*;
use uefi::boot::{self, ScopedProtocol};

// To access measure data variables
const MEASURE_VARIABLES: VariableVendor = VariableVendor(guid!("12345678-1234-1234-1234-123456789012"));

const FILENAME_STATS: &str = "measure_count.txt";
const AMOUNT_OF_RESTARTS: u8 = 100;

fn open_file(
    simple_fs: &mut ScopedProtocol<SimpleFileSystem>,
    filename: &str,
) -> Result<RegularFile, Status> {
    let mut root_dir: Directory = simple_fs.open_volume().map_err(|err| {
        println!("Failed to open the root directory on a volume: {}", err);
        err.status()
    })?;

    let path = CString16::try_from(filename).map_err(|_| {
        println!("Failed to create CString16 from filename.");
        Status::INVALID_PARAMETER
    })?;

    let file_mode = FileMode::CreateReadWrite;
    let file_attr = FileAttribute::empty();

    let file_handle = root_dir.open(path.as_ref(), file_mode, file_attr).map_err(|err| {
        println!("Failed to open a file relative to the root directory: {}", err);
        err.status()
    })?;

    file_handle.into_regular_file().ok_or_else(|| {
        println!("Specified file is not actually a regular file");
        Status::NOT_FOUND
    })
}

fn write_timestamped_entry(
    file: &mut RegularFile,
    count: u8
) -> Result<(), Status> {
    file.set_position(RegularFile::END_OF_FILE).map_err(|err| {
        println!("Failed seeking past the end of the file: {}", err);
        err.status()
    })?;

    let time = uefi::runtime::get_time().map_err(|err| {
        println!("Failed to get current time: {}", err);
        err.status()
    })?;

    // Calculate seconds since midnight
    let seconds: u64 = (time.hour() as u64) * 3600 + (time.minute() as u64) * 60 + (time.second() as u64);
    let text = format!("{count}: {seconds} seconds\n");
    let txt_bytes = text.as_bytes();

    // TODO: check a number of bytes that were actually written
    file.write(txt_bytes).map_err(|err| {
        println!("Failed to write to the file: {}", err);
        err.status()
    })
}

// fn get_count() -> Result {

// }

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap(); // Initialize uefi::helpers

    let mut buffer = [0u8; 1];
    let mut var_name_str = [0u16; 16];
    let var_name = CStr16::from_str_with_buf("MeasureCount", &mut var_name_str).unwrap();

    let attributes = VariableAttributes::NON_VOLATILE | VariableAttributes::BOOTSERVICE_ACCESS;
    let mut measure_count = AMOUNT_OF_RESTARTS;

    match uefi::runtime::get_variable(var_name, &MEASURE_VARIABLES, &mut buffer) {
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

                if let Err(err) = set_variable(var_name, &MEASURE_VARIABLES, attributes, &[]) {
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

            if let Err(err) = set_variable(var_name, &MEASURE_VARIABLES, attributes, &buffer) {
                println!("Error setting variable: {}", err);
                return err.status();
            }
        }
    }

    measure_count -= 1;
    buffer[0] = measure_count;
    if let Err(err) = set_variable(var_name, &MEASURE_VARIABLES, attributes, &buffer) {
        println!("Error setting variable: {}", err);
        return err.status();
    }

    let mut simple_fs: ScopedProtocol<SimpleFileSystem> = match boot::get_image_file_system(boot::image_handle()) {
        Ok(fs) => fs,
        Err(err) => {
            println!("Failed to get image file system: {}", err);
            return err.status();
        }
    };

    let mut file = match open_file(&mut simple_fs, FILENAME_STATS) {
        Ok(f) => f,
        Err(status) => return status,
    };

    if let Err(status) = write_timestamped_entry(&mut file, measure_count) {
        return status;
    }

    file.close();

    reset(ResetType::COLD, Status::SUCCESS, None);
    // Status::SUCCESS
}