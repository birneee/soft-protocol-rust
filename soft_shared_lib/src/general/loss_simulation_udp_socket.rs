use std::net::{UdpSocket, ToSocketAddrs, SocketAddr};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use std::time::Duration;

/// Wraps a normal UdpSocket
///
/// Sent packet are lost with the specified probability
pub struct LossSimulationUdpSocket {
    inner: UdpSocket,
    /// the probability that the next package sent will be lost if the last packet was lost
    p: f64,
    /// the probability that the next package sent will be lost if the last packet was also lost
    q: f64,
    last_packet_lost: AtomicBool,
}

impl LossSimulationUdpSocket {

    /// Creates a UDP socket from the given address.
    ///
    /// if `p` and `q` are 0, the Socket behaves like a normal UdpSocket
    ///
    /// #Argument
    /// * `p` - the probability that the next package sent will be lost if the last packet was lost
    /// * `q` - the probability that the next package sent will be lost if the last packet was also lost
    pub fn bind<A: ToSocketAddrs>(addr: A, p: f64, q: f64) -> std::io::Result<LossSimulationUdpSocket> {
        assert!(0.0 <= p && p <= 1.0);
        assert!(0.0 <= q && q <= 1.0);
        Ok(LossSimulationUdpSocket {
            inner: UdpSocket::bind(addr)?,
            p,
            q,
            last_packet_lost: AtomicBool::new(false),
        })
    }

    /// unmodified receive function
    pub fn recv_from(&self, buf: &mut [u8]) -> std::io::Result<(usize, SocketAddr)> {
        self.inner.recv_from(buf)
    }

    /// modified send function.
    /// Sent packet are lost with the specified probability.
    pub fn send_to<A: ToSocketAddrs>(&self, buf: &[u8], addr: A) -> std::io::Result<usize> {
        if self.random_loss() {
            Ok(buf.len())
        } else {
            self.inner.send_to(buf, addr)
        }
    }

    fn random_loss(&self) -> bool {
        let rand: f64 = rand::random();
        if let Ok(loss) = self.last_packet_lost.compare_exchange(true, rand < self.q, SeqCst, SeqCst) {
            return loss;
        }
        if let Ok(loss) = self.last_packet_lost.compare_exchange(false, rand < self.p, SeqCst, SeqCst) {
            return loss;
        }
        panic!()
    }

    pub fn set_read_timeout(&self, dur: Option<Duration>) -> std::io::Result<()> {
        self.inner.set_read_timeout(dur)
    }

    pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
        self.inner.local_addr()
    }
}