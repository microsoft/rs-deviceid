/// A sample CLI program to retrieve or generate the device id
use deviceid::{DevDeviceId, Result};

enum Command {
    Get,
    Generate,
    Help,
}

fn main() -> Result<()> {
    let arg1 = std::env::args().nth(1);
    let cmd = match (std::env::args().count(), arg1.as_deref()) {
        (1, None) => Command::Get,
        (2, Some("-f")) => Command::Generate,
        (2, Some("-h" | "--help" | "-v" | "--version")) => Command::Help,
        (count, arg1) => {
            eprintln!("count: {count}, arg1: {arg1:?}");
            Command::Help
        }
    };

    match cmd {
        Command::Get => {
            let device_id = DevDeviceId::get()?;

            match device_id {
                Some(id) => println!("Device ID: {}", id),
                None => {
                    let exe = std::env::current_exe().unwrap();
                    let own_name = exe.file_name().unwrap().to_string_lossy();
                    eprintln!(
                        "No Device ID found, generate a new one with '{own_name} -f'",
                        own_name = own_name
                    );
                }
            }
            Ok(())
        }
        Command::Generate => {
            let device_id = DevDeviceId::get_or_generate()?;
            println!("Device ID: {}", device_id);
            Ok(())
        }
        Command::Help => {
            let exe = std::env::current_exe().unwrap();
            let own_name = exe.file_name().unwrap().to_string_lossy();
            println!("Usage: {} [-f] [-h | --help] [-v | --version]", own_name);
            println!("Options:");
            println!("  -f               Generate a new Device ID, if one is not already set");
            println!("  -h, --help       Show this help message");
            println!("  -v, --version    Show version information");
            Ok(())
        }
    }
}
