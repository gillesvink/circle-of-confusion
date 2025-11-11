use std::io::{Result, Write};

fn main() -> Result<()> {
    #[cfg(feature = "compile-protobuf-src")]
    std::env::set_var("PROTOC", protobuf_src::protoc());

    let mut config = prost_build::Config::new();
    #[cfg(feature = "serde")]
    config.type_attribute(".", "#[derive(serde::Serialize,serde::Deserialize)]");

    config.compile_protos(&["circle_of_confusion.proto"], &["proto/"])?;

    write_version();

    Ok(())
}

fn write_version() {
    let path = std::path::Path::new("./version.txt");
    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(
        format!(
            "version = '{}'",
            std::env::var("CARGO_PKG_VERSION").unwrap()
        )
        .as_bytes(),
    )
    .unwrap();
}
