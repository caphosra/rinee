use async_std::{
    io::{prelude::BufReadExt, BufReader, BufWriter, WriteExt},
    net::TcpStream,
};

use crate::{
    parser::parse_request,
    proto::{Error, Request},
    write_log, Args,
};

pub async fn play_game(args: &Args) -> Result<(), Error> {
    let addr = format!("{}:{}", args.host, args.port);
    let stream = TcpStream::connect(addr).await?;
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);

    writer
        .write(format!("OPEN {}\n", args.name).as_bytes())
        .await?;

    let mut buf = String::new();
    reader.read_line(&mut buf).await?;

    let req = parse_request(&buf)?;
    match req {
        Request::Start {
            color,
            opponent,
            remains,
        } => {
            write_log!(
                LOG,
                "Start color={}, opponent={}, remains={}",
                color,
                opponent,
                remains
            );
        }
        _ => {}
    }

    Ok(())
}
