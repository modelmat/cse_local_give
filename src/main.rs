use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use ssh2::Session;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process;
use xdg::BaseDirectories;

use subprocess::Exec;

const NUM_RAND_CHARS: usize = 12;
const SERVER_URL: &str = "login4.cse.unsw.edu.au";

const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");

fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {}", err);
        process::exit(1);
    });
    
    let xdg_dirs = BaseDirectories::with_prefix(PACKAGE_NAME).unwrap();
    let config_path = xdg_dirs.place_config_file("zID").unwrap_or_else(|err| {
        println!("Could not fetch configuration directory: {}", err);
        process::exit(1);
    });
    let mut config_file = File::open(&config_path).unwrap_or_else(|err| {
        println!("Could not open zID config file ({}): {}", config_path.display(), err);
        process::exit(1);
    });
    let mut z_id = String::new();
    config_file.read_to_string(&mut z_id).unwrap_or_else(|err| {
        println!("Could not read zID config file({}): {}", config_path.display(), err);
        process::exit(1);
    });
    let z_id = z_id.trim();

    let local_files_folder = env::current_dir().unwrap();

    // Connect to the SSH server
    let tcp = TcpStream::connect(&format!("{}:22", SERVER_URL)).unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();

    // Try to authenticate with the first identity in the agent.
    sess.userauth_agent(z_id).unwrap();

    // Make sure we succeeded
    assert!(sess.authenticated());

    let mut channel = sess.channel_session().unwrap();
    channel
        .exec(&format!("{} classrun -assigns", config.class))
        .unwrap();
    let mut s = String::new();
    channel.read_to_string(&mut s).unwrap();
    let assignments: Vec<String> = s.lines().map(|s| s.to_string()).collect();
    channel.wait_close().unwrap();

    if !assignments.contains(&config.assignment) {
        println!(
            "Unknown assignment {}, valid assignments are:\n- {}",
            config.assignment,
            assignments.join("\n- ")
        );
        process::exit(1);
    }

    assert_eq!(channel.exit_status().unwrap(), 0);

    let rng = thread_rng();

    let suffix: String = rng
        .sample_iter(&Alphanumeric)
        .take(NUM_RAND_CHARS)
        .map(char::from)
        .collect();
    let path_str = format!("/tmp/{}_submission_{}", z_id, suffix);
    let path = Path::new(&path_str);

    let sftp = sess.sftp().unwrap();
    sftp.mkdir(path, 0o777).unwrap();
    for file in config.files {
        let mut v = Vec::new();
        File::open(&local_files_folder.join(file))
            .unwrap()
            .read_to_end(&mut v)
            .unwrap();
        sftp.create(&path.join(file))
            .unwrap()
            .write_all(&v)
            .unwrap();
    }

    let shell_command = format!(
        "ssh -qt {}@{} \"cd {} && {} classrun -give {} {}\"",
        z_id,
        SERVER_URL,
        path_str,
        config.class,
        config.assignment,
        config.files.join(" ")
    );

    Exec::shell(shell_command).popen().unwrap().wait().unwrap();
}

struct Config<'a> {
    class: String,
    assignment: String,
    files: &'a [String],
}

impl Config<'_> {
    fn new(args: &[String]) -> Result<Config, &'static str> {
        match args.len() {
            0 => panic!(),
            1 => Err("Class code not supplied"),
            2 => Err("Assignment not specified"),
            3 => Err("No files specified"),
            _ => {
                let class = args[1].clone();
                let assignment = args[2].clone();
                let files = &args[3..];
                if files
                    .iter()
                    .all(|file| env::current_dir().unwrap().join(file).exists())
                {
                    Ok(Config {
                        class,
                        assignment,
                        files,
                    })
                } else {
                    Err("One of the provided files do not exist")
                }
            }
        }
    }
}
