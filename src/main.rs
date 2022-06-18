use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use ssh2::Session;
use std::io::Write;
use std::net::TcpStream;
use std::path::Path;

use subprocess::Exec;

const NUM_RAND_CHARS: usize = 12;

fn main() {
    let z_id = "zXXXXXXX";

    // Connect to the SSH server
    let tcp = TcpStream::connect("login4.cse.unsw.edu.au:22").unwrap();
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();

    // Try to authenticate with the first identity in the agent.
    sess.userauth_agent("zXXXXXXX").unwrap();

    // Make sure we succeeded
    assert!(sess.authenticated());

    let class = "2521";
    let assignment = "lab04";
    let files = vec!["BSTree.c", "analysis.txt"];

    let rng = thread_rng();

    let suffix: String = rng
        .sample_iter(&Alphanumeric)
        .take(NUM_RAND_CHARS)
        .map(char::from)
        .collect();
    let path_str = format!("/tmp/{}_submission_{}", z_id, suffix);
    let path = Path::new(&path_str);
    println!("Path: {}", path.display());

    let sftp = sess.sftp().unwrap();
    sftp.mkdir(path, 0o777).unwrap();
    for file in &files {
        sftp.create(&path.join(file))
            .unwrap()
            .write_all(file.as_bytes())
            .unwrap();
    }
    let ret = sftp.readdir(path).unwrap();
    for (path, _) in ret {
        println!(
            "{}",
            path.display() /*file_name().unwrap().to_string_lossy()*/
        );
    }

    let shell_command = format!(
        "ssh -qt {}@cse.unsw.edu.au \"cd {} && {} classrun -give {} {}\"",
        z_id,
        path_str,
        class,
        assignment,
        files.join(" ")
    );
    println!("Command: {}", shell_command);
    Exec::shell(shell_command).popen().unwrap().wait().unwrap();
}
