//! Dependency-free, localhost-only RFC 6455 echo harness.
//! This exercises a Rust HOST transport, NOT an AttoLang actor or guest.
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

const MAX_MESSAGE: usize = 65536;
const MAX_CONNECTIONS: usize = 16;
const GUID: &[u8] = b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn sha1(input: &[u8]) -> [u8; 20] {
    let mut data = input.to_vec();
    data.push(128);
    while data.len() % 64 != 56 { data.push(0); }
    data.extend_from_slice(&((input.len() as u64).wrapping_mul(8)).to_be_bytes());
    let mut h = [0x67452301u32,0xefcdab89,0x98badcfe,0x10325476,0xc3d2e1f0];
    for chunk in data.chunks_exact(64) {
        let mut w = [0u32; 80];
        for (i,v) in w.iter_mut().enumerate().take(16) {
            *v = u32::from_be_bytes(chunk[i*4..i*4+4].try_into().unwrap());
        }
        for i in 16..80 { w[i] = (w[i-3]^w[i-8]^w[i-14]^w[i-16]).rotate_left(1); }
        let (mut a,mut b,mut c,mut d,mut e) = (h[0],h[1],h[2],h[3],h[4]);
        for (i,word) in w.iter().enumerate() {
            let (f,k) = match i {
                0..=19 => ((b&c)|(!b&d), 0x5a827999u32),
                20..=39 => (b^c^d, 0x6ed9eba1),
                40..=59 => ((b&c)|(b&d)|(c&d), 0x8f1bbcdc),
                _ => (b^c^d, 0xca62c1d6),
            };
            let t=a.rotate_left(5).wrapping_add(f).wrapping_add(e).wrapping_add(k).wrapping_add(*word);
            e=d; d=c; c=b.rotate_left(30); b=a; a=t;
        }
        for (old,now) in h.iter_mut().zip([a,b,c,d,e]) { *old=old.wrapping_add(now); }
    }
    let mut output=[0u8;20];
    for (i,v) in h.iter().enumerate() { output[4*i..4*i+4].copy_from_slice(&v.to_be_bytes()); }
    output
}
fn b64(bytes: &[u8]) -> String {
    let mut out=String::new();
    for c in bytes.chunks(3) {
        let a=c[0]; let b=*c.get(1).unwrap_or(&0); let d=*c.get(2).unwrap_or(&0);
        out.push(B64[(a>>2) as usize] as char);
        out.push(B64[(((a&3)<<4)|(b>>4)) as usize] as char);
        out.push(if c.len()>1 { B64[(((b&15)<<2)|(d>>6)) as usize] as char } else {'='});
        out.push(if c.len()>2 { B64[(d&63) as usize] as char } else {'='});
    }
    out
}
fn valid_key(s: &str) -> bool {
    let b=s.as_bytes();
    if b.len()!=24 || &b[22..]!=b"==" { return false; }
    let mut last=0usize;
    for (i,v) in b[..22].iter().enumerate() {
        let Some(index)=B64.iter().position(|x|x==v) else {return false};
        if i==21 {last=index;}
    }
    last&15==0 // 16 canonical decoded bytes
}
fn local_host(s: &str) -> bool {
    let Some((host,port))=s.rsplit_once(':') else {return false};
    (host=="localhost"||host=="127.0.0.1") && port.parse::<u16>().is_ok_and(|p|p>0)
}
fn upgrade(request: &str) -> Result<String,&'static str> {
    if !request.ends_with("\r\n\r\n") {return Err("incomplete upgrade");}
    let mut lines=request[..request.len()-4].split("\r\n");
    if lines.next()!=Some("GET /ws HTTP/1.1") {return Err("only GET /ws HTTP/1.1");}
    let (mut host,mut up,mut conn,mut version,mut key,mut origin)=(None,None,None,None,None,None);
    for line in lines {
        let (name,value)=line.split_once(':').ok_or("malformed header")?;
        if name.is_empty()||!name.bytes().all(|c|c.is_ascii_alphanumeric()||c==b'-') {
            return Err("invalid header name");
        }
        if value.bytes().any(|c|c<32&&c!=9 || c==127) {return Err("invalid header value");}
        let val=value.trim();
        let slot = match name.to_ascii_lowercase().as_str() {
            "host"=>&mut host,
            "upgrade"=>&mut up,
            "connection"=>&mut conn,
            "sec-websocket-version"=>&mut version,
            "sec-websocket-key"=>&mut key,
            "origin"=>&mut origin,
            "sec-websocket-extensions"|"sec-websocket-protocol"=>return Err("unsupported negotiation"),
            "content-length"|"transfer-encoding"=>return Err("HTTP body framing not supported for upgrade"),
            _=>continue,
        };
        if slot.replace(val).is_some() {return Err("duplicate protected header");}
    }
    if !host.is_some_and(local_host) {return Err("nonlocal Host");}
    if !up.is_some_and(|s:&str|s.eq_ignore_ascii_case("websocket")) {return Err("invalid Upgrade");}
    if !conn.is_some_and(|s:&str|s.split(',').any(|v|v.trim().eq_ignore_ascii_case("upgrade"))) {
        return Err("invalid Connection");
    }
    if version!=Some("13") {return Err("unsupported version");}
    if origin.is_some_and(|s:&str|{
        let Some(authority)=s.strip_prefix("http://").or_else(||s.strip_prefix("https://")) else{return true};
        !local_host(authority)
    }) {return Err("cross-origin denied");}
    let key=key.ok_or("missing key")?;
    if !valid_key(key) {return Err("invalid key");}
    let mut material=key.as_bytes().to_vec();
    material.extend_from_slice(GUID);
    Ok(b64(&sha1(&material)))
}
fn read_upgrade(stream: &mut TcpStream) -> io::Result<String> {
    read_upgrade_with_deadline(stream,Duration::from_secs(10))
}
fn read_upgrade_with_deadline(stream: &mut TcpStream, budget:Duration) -> io::Result<String> {
    let deadline=Instant::now()+budget;
    let mut bytes=Vec::new();
    for _ in 0..8192 {
        let remaining=deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(io::Error::new(io::ErrorKind::TimedOut,"handshake deadline exceeded"));
        }
        stream.set_read_timeout(Some(remaining))?;
        let mut b=[0u8;1]; stream.read_exact(&mut b)?; bytes.push(b[0]);
        if bytes.ends_with(b"\r\n\r\n") {
            return String::from_utf8(bytes).map_err(|_|io::Error::new(io::ErrorKind::InvalidData,"non-UTF8 header"));
        }
    }
    Err(io::Error::new(io::ErrorKind::InvalidData,"header budget exceeded"))
}
#[derive(Debug,PartialEq,Eq)]
struct Frame {fin:bool, opcode:u8, data:Vec<u8>}
fn read_frame(reader:&mut impl Read)->Result<Frame,u16>{
    let mut h=[0u8;2]; reader.read_exact(&mut h).map_err(|_|1002)?;
    if h[0]&0x70!=0 || h[1]&0x80==0 {return Err(1002);} // RSV, masking
    let fin=h[0]&128!=0; let opcode=h[0]&15;
    if !matches!(opcode,0|1|2|8|9|10) {return Err(1002);}
    let code=(h[1]&127) as usize;
    let len=match code {
        126=>{
            let mut b=[0u8;2]; reader.read_exact(&mut b).map_err(|_|1002)?;
            let n=u16::from_be_bytes(b) as usize;
            if n<126 {return Err(1002);} n
        },
        127=>{
            let mut b=[0u8;8]; reader.read_exact(&mut b).map_err(|_|1002)?;
            if b[0]&128!=0 {return Err(1002);}
            let n=u64::from_be_bytes(b);
            if n<=65535 {return Err(1002);} // reject nonminimal 64-bit length
            if n>MAX_MESSAGE as u64 {return Err(1009);}
            n as usize
        },
        _=>code,
    };
    if len>MAX_MESSAGE {return Err(1009);}
    if opcode>=8 && (!fin||len>125) {return Err(1002);}
    let mut mask=[0u8;4]; reader.read_exact(&mut mask).map_err(|_|1002)?;
    let mut data=vec![0u8;len]; reader.read_exact(&mut data).map_err(|_|1002)?;
    for (i,b) in data.iter_mut().enumerate(){*b^=mask[i%4];}
    Ok(Frame{fin,opcode,data})
}
fn write_frame(writer:&mut impl Write, opcode:u8, data:&[u8])->io::Result<()>{
    if data.len()>MAX_MESSAGE {return Err(io::Error::new(io::ErrorKind::InvalidInput,"oversized reply"));}
    let mut h=vec![0x80|opcode];
    if data.len()<126 {h.push(data.len() as u8);}
    else if data.len()<=u16::MAX as usize {
        h.push(126); h.extend_from_slice(&(data.len() as u16).to_be_bytes());
    } else {
        h.push(127); h.extend_from_slice(&(data.len() as u64).to_be_bytes());
    }
    writer.write_all(&h)?; writer.write_all(data)?; writer.flush()
}
fn close(writer:&mut impl Write,code:u16) {let _=write_frame(writer,8,&code.to_be_bytes());}
fn close_code_ok(c:u16)->bool {matches!(c,1000..=1003|1007..=1014|3000..=4999)}
fn echo(writer:&mut impl Write,op:u8,data:&[u8])->Result<(),u16>{
    if op==1&&std::str::from_utf8(data).is_err(){return Err(1007);}
    write_frame(writer,op,data).map_err(|_|1011)
}
fn handle(mut stream:TcpStream)->io::Result<()>{
    stream.set_read_timeout(Some(Duration::from_secs(10)))?;
    stream.set_write_timeout(Some(Duration::from_secs(10)))?;
    let accept=match read_upgrade(&mut stream).and_then(|s|upgrade(&s)
        .map_err(|msg|io::Error::new(io::ErrorKind::InvalidData,msg))) {
        Ok(v)=>v,
        Err(_)=>{
            let _=stream.write_all(b"HTTP/1.1 400 Bad Request\r\nConnection: close\r\nContent-Length: 0\r\n\r\n");
            return Ok(());
        },
    };
    stream.write_all(format!("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {accept}\r\n\r\n").as_bytes())?;
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    let mut partial:Option<(u8,Vec<u8>)>=None;
    let mut fragment_started:Option<Instant>=None;
    loop {
        if fragment_started.is_some_and(|start|start.elapsed()>=Duration::from_secs(30)) {
            close(&mut stream,1008);return Ok(());
        }
        let f=match read_frame(&mut stream){Ok(f)=>f,Err(code)=>{close(&mut stream,code);return Ok(());}};
        match f.opcode {
            8=>{
                if f.data.len()==1 {close(&mut stream,1002);return Ok(());}
                if f.data.len()>=2 {
                    let c=u16::from_be_bytes([f.data[0],f.data[1]]);
                    if !close_code_ok(c){close(&mut stream,1002);return Ok(());}
                    if std::str::from_utf8(&f.data[2..]).is_err(){close(&mut stream,1007);return Ok(());}
                }
                write_frame(&mut stream,8,&f.data)?;return Ok(());
            },
            9=>write_frame(&mut stream,10,&f.data)?,
            10=>{},
            0=>{
                let Some((_,buf))=partial.as_mut() else {close(&mut stream,1002);return Ok(());};
                if buf.len().checked_add(f.data.len()).is_none_or(|n|n>MAX_MESSAGE) {
                    close(&mut stream,1009);return Ok(());
                }
                buf.extend_from_slice(&f.data);
                if f.fin {
                    let (op,data)=partial.take().unwrap();
                    fragment_started=None;
                    if let Err(code)=echo(&mut stream,op,&data){close(&mut stream,code);return Ok(());}
                }
            },
            1|2=>{
                if partial.is_some(){close(&mut stream,1002);return Ok(());}
                if f.fin {
                    if let Err(code)=echo(&mut stream,f.opcode,&f.data){close(&mut stream,code);return Ok(());}
                }else{
                    fragment_started=Some(Instant::now());
                    partial=Some((f.opcode,f.data));
                }
            },
            _=>unreachable!(),
        }
    }
}
struct Slot(Arc<AtomicUsize>);
impl Drop for Slot{fn drop(&mut self){self.0.fetch_sub(1,Ordering::AcqRel);}}
fn serve()->io::Result<()>{
    let listener=TcpListener::bind("127.0.0.1:9001")?;
    let slots=Arc::new(AtomicUsize::new(0));
    println!("Host-only Rust WebSocket echo: ws://127.0.0.1:9001/ws; AttoLang guest NOT_EXECUTED");
    for connection in listener.incoming(){
        let mut stream=connection?;
        if slots.fetch_update(Ordering::AcqRel,Ordering::Acquire,|n|
            if n<MAX_CONNECTIONS {Some(n+1)}else{None}).is_err(){
            let _=stream.write_all(b"HTTP/1.1 503 Service Unavailable\r\nConnection: close\r\nContent-Length: 0\r\n\r\n");
            continue;
        }
        let count=Arc::clone(&slots);
        std::thread::spawn(move||{
            let _token=Slot(count);
            if let Err(e)=handle(stream){eprintln!("client I/O: {e}");}
        });
    }
    Ok(())
}
fn main(){if let Err(e)=serve(){eprintln!("server: {e}");std::process::exit(1);}}

#[cfg(test)]
mod tests{
    use super::*;
    use std::io::Cursor;
    fn request(port:u16)->String {
        format!("GET /ws HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nUpgrade: websocket\r\nConnection: keep-alive, Upgrade\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n")
    }
    fn masked(op:u8,fin:bool,data:&[u8])->Vec<u8>{
        assert!(data.len()<=125);
        let mask=[1,2,3,4];
        let mut out=vec![(if fin{128}else{0})|op,128|data.len() as u8];
        out.extend_from_slice(&mask);
        for (i,b) in data.iter().enumerate(){out.push(b^mask[i%4]);}
        out
    }
    #[test] fn rfc6455_handshake_vector(){
        assert_eq!(upgrade(&request(9001)),Ok("s3pPLMBiTxaQ9kYGzzhZRbK+xOo=".into()));
    }
    #[test] fn header_negative_cases(){
        let q=request(9001);
        assert!(upgrade(&q.replace("/ws","/nope")).is_err());
        assert!(upgrade(&q.replace("Version: 13","Version: 12")).is_err());
        assert!(upgrade(&q.replace("ZQ==","ZZ==")).is_err());
        assert!(upgrade(&q.replace("\r\n\r\n","\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n")).is_err());
        assert!(upgrade(&q.replace("\r\n\r\n","\r\nOrigin: https://evil.test\r\n\r\n")).is_err());
        assert!(upgrade(&q.replace("\r\n\r\n","\r\nSec-WebSocket-Extensions: permessage-deflate\r\n\r\n")).is_err());
        assert!(upgrade(&q.replace("127.0.0.1:9001","evil.test")).is_err());
        assert!(upgrade(&q.replace("\r\n\r\n","\r\nContent-Length: 0\r\n\r\n")).is_err());
        assert!(upgrade(&q.replace("\r\n\r\n","\r\nTransfer-Encoding: chunked\r\n\r\n")).is_err());
    }
    #[test] fn frame_negative_cases(){
        assert_eq!(read_frame(&mut Cursor::new([0x81,1,b'x'])),Err(1002));
        let mut rsv=masked(1,true,b"x");rsv[0]|=0x40;
        assert_eq!(read_frame(&mut Cursor::new(rsv)),Err(1002));
        assert_eq!(read_frame(&mut Cursor::new(masked(9,false,b"x"))),Err(1002));
        assert_eq!(read_frame(&mut Cursor::new(masked(3,true,b"x"))),Err(1002));
        assert_eq!(read_frame(&mut Cursor::new([0x81,0xfe,0,125])),Err(1002));
        assert_eq!(read_frame(&mut Cursor::new([0x82,0xff,0,0,0,0,0,1,0,1])),Err(1009));
        assert_eq!(read_frame(&mut Cursor::new([0x82,0xff,0x80,0,0,0,0,1,0,0])),Err(1002));
        assert_eq!(read_frame(&mut Cursor::new([0x82,0xff,0,0,0,0,0,0,0,125])),Err(1002));
        assert_eq!(read_frame(&mut Cursor::new(masked(1,true,b"ok"))).unwrap().data,b"ok");
        assert_eq!(echo(&mut Vec::new(),1,&[255]),Err(1007));
        assert!(!close_code_ok(1005));assert!(!close_code_ok(2000));
    }


    #[test] fn stalled_handshake_obeys_absolute_deadline(){
        let listener=TcpListener::bind("127.0.0.1:0").unwrap();
        let address=listener.local_addr().unwrap();
        let peer=TcpStream::connect(address).unwrap();
        let (mut accepted,_)=listener.accept().unwrap();
        peer.set_nodelay(true).unwrap();
        let start=Instant::now();
        let error=read_upgrade_with_deadline(&mut accepted,Duration::from_millis(200)).unwrap_err();
        assert!(matches!(error.kind(),io::ErrorKind::TimedOut|io::ErrorKind::WouldBlock));
        assert!(start.elapsed()<Duration::from_secs(3));
    }

    #[test] fn exact_64k_frame_encoding_and_masked_decoding(){
        let data=vec![0x4du8;MAX_MESSAGE];
        let mut output=Vec::new();
        write_frame(&mut output,2,&data).unwrap();
        assert_eq!(&output[..10],&[0x82,127,0,0,0,0,0,1,0,0]);
        assert_eq!(&output[10..],data.as_slice());
        let mask=[0x11u8,0x22,0x33,0x44];
        let mut inbound=vec![0x82,255];
        inbound.extend_from_slice(&(MAX_MESSAGE as u64).to_be_bytes());
        inbound.extend_from_slice(&mask);
        for (i,v) in data.iter().enumerate(){inbound.push(v^mask[i%4]);}
        let frame=read_frame(&mut Cursor::new(inbound)).unwrap();
        assert!(frame.fin);
        assert_eq!(frame.opcode,2);
        assert_eq!(frame.data,data);
        assert!(write_frame(&mut Vec::new(),2,&vec![0;MAX_MESSAGE+1]).is_err());
    }
    #[test] fn invalid_peer_frame_causes_protocol_close(){
        for (frame,expected) in [
            (vec![0x81,1,b'x'],1002u16),
            (masked(0,true,b"stray"),1002u16),
            (masked(1,true,&[0xff]),1007u16),
            (masked(8,true,&[0x03]),1002u16),
        ] {
            let listener=TcpListener::bind("127.0.0.1:0").unwrap();
            let address=listener.local_addr().unwrap();
            let server=std::thread::spawn(move||{
                let (peer,_)=listener.accept().unwrap();
                handle(peer).unwrap();
            });
            let mut peer=TcpStream::connect(address).unwrap();
            peer.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
            peer.write_all(request(address.port()).as_bytes()).unwrap();
            let mut header=Vec::new();
            while !header.ends_with(b"\r\n\r\n"){
                let mut one=[0u8;1];
                peer.read_exact(&mut one).unwrap();
                header.push(one[0]);
                assert!(header.len()<1024);
            }
            assert!(header.starts_with(b"HTTP/1.1 101"));
            peer.write_all(&frame).unwrap();
            let mut close=[0u8;4];
            peer.read_exact(&mut close).unwrap();
            assert_eq!(&close[..2],&[0x88,2]);
            assert_eq!(u16::from_be_bytes([close[2],close[3]]),expected);
            server.join().unwrap();
        }
    }

    #[test] fn tcp_loopback_fragment_ping_echo_close(){
        let listener=TcpListener::bind("127.0.0.1:0").unwrap();
        let addr=listener.local_addr().unwrap();
        let server=std::thread::spawn(move||{
            let (peer,_)=listener.accept().unwrap();
            handle(peer).unwrap();
        });
        let mut peer=TcpStream::connect(addr).unwrap();
        peer.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
        peer.write_all(request(addr.port()).as_bytes()).unwrap();
        let mut reply=Vec::new();
        while !reply.ends_with(b"\r\n\r\n") {
            let mut ch=[0u8;1];peer.read_exact(&mut ch).unwrap();reply.push(ch[0]);
            assert!(reply.len()<1024);
        }
        assert!(String::from_utf8(reply).unwrap().contains("101 Switching Protocols"));
        peer.write_all(&masked(1,false,b"he")).unwrap();
        peer.write_all(&masked(9,true,b"p")).unwrap();
        let mut pong=[0u8;3];peer.read_exact(&mut pong).unwrap();
        assert_eq!(pong,[0x8a,1,b'p']);
        peer.write_all(&masked(0,true,b"llo")).unwrap();
        let mut echoed=[0u8;7];peer.read_exact(&mut echoed).unwrap();
        assert_eq!(echoed,[0x81,5,b'h',b'e',b'l',b'l',b'o']);
        peer.write_all(&masked(8,true,&1000u16.to_be_bytes())).unwrap();
        let mut closed=[0u8;4];peer.read_exact(&mut closed).unwrap();
        assert_eq!(closed,[0x88,2,3,232]);
        server.join().unwrap();
    }
}
