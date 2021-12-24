use std::fmt;
use std::io;
use ureq::{Error, ReadWrite, TlsConnector};

use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use mbedtls::ssl::config::{Endpoint, Preset, Transport};
use mbedtls::ssl::{Config, Context};
use mbedtls::rng::CtrDrbg;

fn entropy_new() -> mbedtls::rng::OsEntropy {
    mbedtls::rng::OsEntropy::new()
}

pub struct MbedTlsConnector {
    context:  Arc<Mutex<Context>>
}

#[derive(Debug)]
struct MbedTlsError;
impl fmt::Display for MbedTlsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MedTLS handshake failed")
    }
}

impl std::error::Error for MbedTlsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[allow(dead_code)]
pub(crate) fn default_tls_config() -> std::sync::Arc<dyn TlsConnector> {
    Arc::new(MbedTlsConnector::new(mbedtls::ssl::config::AuthMode::Required))
}

impl MbedTlsConnector {
    pub fn new(mode: mbedtls::ssl::config::AuthMode) -> MbedTlsConnector {
        let entropy = Arc::new(entropy_new());
        let mut config = Config::new(Endpoint::Client, Transport::Stream, Preset::Default);
        let rng = Arc::new(CtrDrbg::new(entropy, None).unwrap());
        config.set_rng(rng);
        config.set_authmode(mode);
        let ctx = Context::new(Arc::new(config));
        MbedTlsConnector {
            context: Arc::new(Mutex::new(ctx))
        }
    }
}

impl TlsConnector for MbedTlsConnector {
    fn connect(
        &self,
        _dns_name: &str,
        tcp_stream: TcpStream,
    ) -> Result<Box<dyn ReadWrite>, Error> {

        let mut ctx = self.context.lock().unwrap();
        match ctx.establish(tcp_stream, None) {
            Err(_) => {
                let io_err = io::Error::new(io::ErrorKind::InvalidData, MbedTlsError);
                return Err(io_err.into());
            }
            Ok(()) => Ok(MbedTlsStream::new(self))
        }
    }
}

struct MbedTlsStream {
    context:  Arc<Mutex<Context>>
    //tcp_stream: TcpStream,
}

impl MbedTlsStream {
    pub fn new(mtc: &MbedTlsConnector) -> Box<MbedTlsStream> {
        Box::new(MbedTlsStream {
            context: mtc.context.clone()
        })
    }
}


impl ReadWrite for MbedTlsStream {
    fn socket(&self) -> Option<&TcpStream> {
        None
    }
}

impl io::Read for MbedTlsStream  {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut ctx = self.context.lock().unwrap();
        ctx.read(buf)
    }
}

impl io::Write for MbedTlsStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut ctx = self.context.lock().unwrap();
        ctx.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut ctx = self.context.lock().unwrap();
        ctx.flush()
    }
}


/*
 * Local Variables:
 * compile-command: "cd ../.. && cargo build --example mbedtls-req --features=\"mbedtls\""
 * mode: rust
 * End:
 */