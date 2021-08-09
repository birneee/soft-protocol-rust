use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::SeqCst;
use tokio::net::{UdpSocket, ToSocketAddrs};
use std::net::SocketAddr;
use std::io::Result;

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
    pub async fn bind<A: ToSocketAddrs>(addr: A, p: f64, q: f64) -> Result<Self> {
        assert!(0.0 <= p && p <= 1.0);
        assert!(0.0 <= q && q <= 1.0);
        Ok(Self {
            inner: UdpSocket::bind(addr).await?,
            p,
            q,
            last_packet_lost: AtomicBool::new(false),
        })
    }

    /// unmodified receive function
    pub async fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, SocketAddr)> {
        self.inner.recv_from(buf).await
    }

    /// modified send function.
    /// Sent packet are lost with the specified probability.
    pub async fn send_to<A: ToSocketAddrs>(&self, buf: &[u8], addr: A) -> Result<usize> {
        if self.random_loss() {
            Ok(buf.len())
        } else {
            self.inner.send_to(buf, addr).await
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

    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.inner.local_addr()
    }
}