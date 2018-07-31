extern crate rs_release;

use std::process::Command;

#[derive(Debug)]
enum Error {
    UnknownOs,
    ReadError,
}

fn get_os_id() -> Result<String, Error> {
    let os_release = rs_release::get_os_release().map_err(|_| Error::ReadError)?;

    os_release
        .filter_map(Result::ok)
        .find(|(key, _)| key == "ID")
        .map(|(_, value)| value)
        .ok_or(Error::UnknownOs)
}

// https://blog.tinned-software.net/show-installed-yum-packages-by-size/
fn show_fedora_packages() {
    let mut command = Command::new("rpm");

    command.arg("--query");
    command.arg("--all");
    command.arg("--queryformat");
    command.arg("%10{size} - %-25{name} \t %{version}\n");

    if let Err(e) = command.spawn() {
        println!("ERROR running rpm: {:?}", e);
    }
}

// http://www.commandlinefu.com/commands/view/3842/list-your-largest-installed-packages-on-debianubuntu
fn show_debian_packages() {
    let mut command = Command::new("dpkg-query");

    command.arg("--show");
    command.arg("--showformat");
    command.arg("${Installed-Size}\t${Package}\n");

    if let Err(e) = command.spawn() {
        println!("ERROR running dpkg-query: {:?}", e);
    }
}

fn main() {
    match get_os_id() {
        Ok(id) => {
            match id.as_str() {
                "fedora" => show_fedora_packages(),
                "debian" => show_debian_packages(),
                _ => println!("ERROR: {:?}", Error::UnknownOs),
            }
        }
        Err(e) => println!("ERROR: {:?}", e),
    }
}
