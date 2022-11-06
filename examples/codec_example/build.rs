use std::io;

fn main() -> io::Result<()> {
    prost_build::Config::new()
        .out_dir("src/pb")
        .compile_protos(&["proto/foo.proto"], &[""])?;
    Ok(())
}
