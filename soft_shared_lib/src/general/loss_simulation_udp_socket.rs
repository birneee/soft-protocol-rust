use std::net::{UdpSocket, ToSocketAddrs, SocketAddr};
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::sync::atomic::Ordering::SeqCst;
use std::time::Duration;
use core::mem;

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
    packet_losses: AtomicU32
}

impl LossSimulationUdpSocket {

    /// Creates a UDP socket from the given address.
    ///
    /// if `p` and `q` are 0, the Socket behaves like a normal UdpSocket
    ///
    /// #Argument
    /// * `p` - the probability that the next package sent will be lost if the last packet was lost
    /// * `q` - the probability that the next package sent will be lost if the last packet was also lost
    pub fn bind<A: ToSocketAddrs>(addr: A, p: f64, q: f64) -> std::io::Result<Self> {
        assert!(0.0 <= p && p <= 1.0);
        assert!(0.0 <= q && q <= 1.0);
        Ok(Self {
            inner: UdpSocket::bind(addr)?,
            p,
            q,
            last_packet_lost: AtomicBool::new(false),
            packet_losses: AtomicU32::new(0)
        })
    }

    /// unmodified connect function
    pub fn connect(&self, addr: SocketAddr) -> std::io::Result<()> {
        self.inner.connect(addr)
    }

    /// unmodified receive function
    pub fn recv(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.recv(buf)
    }

    /// unmodified receive function
    pub fn recv_from(&self, buf: &mut [u8]) -> std::io::Result<(usize, SocketAddr)> {
        self.inner.recv_from(buf)
    }

    /// modified send function.
    /// Sent packet are lost with the specified probability.
    pub fn send(&self, buf: &[u8]) -> std::io::Result<usize> {
        if self.random_loss() {
            self.packet_losses.fetch_add(1, SeqCst);
            Ok(buf.len())
        } else {
            self.inner.send(buf)
        }
    }

    /// modified send function.
    /// Sent packet are lost with the specified probability.
    pub fn send_to<A: ToSocketAddrs>(&self, buf: &[u8], addr: A) -> std::io::Result<usize> {
        if self.random_loss() {
            self.packet_losses.fetch_add(1, SeqCst);
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

    pub fn peer_addr(&self) -> std::io::Result<SocketAddr> {self.inner.peer_addr()}

    pub fn try_clone(&self) -> std::io::Result<Self> {
        let socket = self.inner.try_clone()?;

        Ok(Self {
            inner: socket,
            p: self.p,
            q: self.q,
            last_packet_lost: AtomicBool::new(self.last_packet_lost.load(SeqCst)),
            packet_losses: AtomicU32::new(self.packet_losses.load(SeqCst))
        })
    }


    pub fn swap_socket(&mut self, new_socket: UdpSocket) -> UdpSocket {
        mem::replace(&mut self.inner, new_socket)
    }

}

impl Drop for LossSimulationUdpSocket {
    fn drop(&mut self) {
        let packet_losses = self.packet_losses.load(SeqCst);
        if packet_losses != 0 {
            log::debug!("Simulated Packet Losses: {}", packet_losses);
        }   
    }
}