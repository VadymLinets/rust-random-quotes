fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .out_dir("src/server/proto")
        .compile_protos(&["proto/quotes.proto"], &["proto"])?;
    Ok(())
}
