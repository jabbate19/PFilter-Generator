use std::collections::HashMap;
use std::fs;
use std::io::{stdin, stdout, Write};

#[derive(Debug)]
struct Permissions {
    ip: String,
    ports: Vec<String>,
    allow_icmp: bool,
}

fn print_logo() {
    println!("\x1b[36m,------.                  ,--.         ,-----.         ,--. \x1b[0m");
    println!("\x1b[36m|  .--. ' ,---. ,--.,--.,-'  '-. ,---. |  |) /_  ,---. `--' \x1b[0m");
    println!("\x1b[36m|  '--'.'| .-. ||  ||  |'-.  .-'| .-. :|  .-.  \\| .-. |,--. \x1b[0m");
    println!("\x1b[36m|  |\\  \\ ' '-' ''  ''  '  |  |  \\   --.|  '--' /' '-' '|  | \x1b[0m");
    println!("\x1b[36m`--' '--' `---'  `----'   `--'   `----'`------'  `---' `--' \x1b[0m");
}

fn yes_no(question: &str) -> bool {
    loop {
        let mut response = String::new();
        print!("{} (y/n)?: ", question);
        let _ = stdout().flush();
        stdin().read_line(&mut response).unwrap();
        if response.len() == 0 {
            continue;
        }
        let first = response.to_lowercase().chars().next().unwrap();
        if first == 'y' {
            return true;
        }
        if first == 'n' {
            return false;
        }
    }
}

fn main() {
    let DEFAULT_SERVICES: HashMap<String, Vec<String>> = HashMap::from([
        (
            String::from("AD"),
            Vec::from([
                String::from("9389"),
                String::from("3289"),
                String::from("3268"),
                String::from("389"),
                String::from("636"),
                String::from("500"),
                String::from("4500"),
                String::from("135"),
                String::from("445"),
            ]),
        ),
        (
            String::from("DNS"),
            Vec::from([
                String::from("53"),
            ]),
        ),
        (
            String::from("HTTP"),
            Vec::from([
                String::from("80"),
                String::from("443"),
            ]),
        ),
        (
            String::from("LDAP"),
            Vec::from([
                String::from("389"),
                String::from("636"),
            ]),
        ),
        (
            String::from("NTP"),
            Vec::from([
                String::from("123"),
            ]),
        ),
        (
            String::from("SMTP"),
            Vec::from([
                String::from("25"),
            ]),
        ),
        (
            String::from("SSH"),
            Vec::from([
                String::from("22"),
            ]),
        ),
        (
            String::from("WinRM"),
            Vec::from([
                String::from("5985"),
                String::from("5986"),
            ]),
        ),
    ]);

    print_logo();
    let mut perms: Vec<Permissions> = Vec::new();
    loop {
        let mut perm = Permissions {
            ip: String::new(),
            ports: Vec::new(),
            allow_icmp: false,
        };
        print!("Enter IP Address: ");
        let _ = stdout().flush();
        stdin().read_line(&mut perm.ip).unwrap();
        perm.ip = perm.ip.trim().to_owned();
        loop {
            let mut port = String::new();
            print!("Enter Port/Common Service to Allow, '?', or nothing to stop: ");
            let _ = stdout().flush();
            stdin().read_line(&mut port).unwrap();
            port = port.trim().to_owned();
            if port.len() == 0 {
                break;
            }
            match port.parse::<u16>() {
                Ok(num) => {
                    if num > 0 {
                        perm.ports.push(port);
                    } else {
                        println!("Invalid Number!");
                    }
                }
                Err(_) => {
                    if port.chars().next().unwrap() == '?' {
                        for (service, ports) in &DEFAULT_SERVICES {
                            println!("{} - {:?}", service, ports);
                        }
                        continue;
                    }
                    match DEFAULT_SERVICES.get(&port) {
                        Some(service_ports) => {
                            for service_port in service_ports {
                                perm.ports.push(service_port.to_owned());
                            }
                        },
                        None => {
                            println!("Service Not Found!");
                        }
                    }
                }
            }
        }
        perm.allow_icmp = yes_no("Allow ICMP");

        println!("{:?}", perm);
        perms.push(perm);

        if yes_no("Add Another Device") {
            continue;
        } else {
            break;
        }
    }
    let mut output = String::from("block all\n\n");
    for perm in perms {
        output.push_str(&format!("#### {}\n", perm.ip));
        for port in perm.ports {
            output.push_str(&format!("### Port {}\n", port));
            output.push_str(&format!(
                "pass in quick proto {{ udp tcp }} from any to {} port {{ {} }}\n",
                perm.ip, port
            ));
            output.push_str(&format!(
                "pass out quick proto {{ udp tcp }} from {} to any port {{ {} }}\n",
                perm.ip, port
            ));
        }
        if perm.allow_icmp {
            output.push_str("### ICMP\n");
            output.push_str(&format!(
                "pass in quick proto {{ icmp }} from any to {}\n",
                perm.ip
            ));
            output.push_str(&format!(
                "pass out quick proto {{ icmp }} from {} to any\n",
                perm.ip
            ));
        }
    }
    output.push_str("pass out proto {{ tcp udp }} from any to port {{ 22 53 80 123 443 }}\n");
    fs::write("/etc/pf.conf", output).unwrap();
}
