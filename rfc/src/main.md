%%%
title = "SOFT Protocol (Draft)"
abbrev = "SOFT Protocol"
ipr= "trust200902"
area = "Transport"
workgroup = "Group 0, 6, 10"
submissiontype = "IETF"
keyword = ["FTP", "TCP"]
date = 2021-05-25T00:00:00Z

[seriesInfo]
name = "RFC"
value = "?"
stream = "IETF"
status = "experimental"

[[author]]
initials = "B."
surname = "Spies"
fullname = "Benedikt Spies"
organization = "Technische Universität München"
[author.address]
email = "benedikt.spies@tum.de"

[[author]]
initials = "T."
surname = "Midek"
fullname = "Thomas Midek"
organization = "Technische Universität München"

[[author]]
initials = "V."
surname = "Giridharan⁩"
fullname = "Vyas Giridharan⁩"
organization = "Technische Universität München"

[[author]]
initials = "D."
surname = "Wilhelm⁩"
fullname = "David Wilhelm⁩"
organization = "Technische Universität München"

[[author]]
initials = "N."
surname = "Pett⁩"
fullname = "Nathalie Pett⁩"
organization = "Technische Universität München"

[[author]]
initials = "F."
surname = "Gust⁩"
fullname = "Felix Gust⁩"
organization = "Technische Universität München"

[[author]]
initials = "M."
surname = "Wiesholler⁩"
fullname = "Maximilian Wiesholler⁩"
organization = "Technische Universität München"
%%%

.# Abstract
The SOFT (Simple One File Transfer) protocol is a ...

{mainmatter}

# Introduction

## Requirements Language

The keywords **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL**, when they appear in this document, are to be interpreted as described in [@RFC2119].

## Terminology


| Term | Description |
| -------- | -------- |
| Client     | Entity requesting one or more files from the server|
| Server     | Entity providing one or more files to the client|
| Connection     | A connection is identified by a unique connection id and compromises all interaction necessary to transfer a single file.|
| Packet     | An SOFT packet is compromised of a header and some payload. There are different types of packets for different purposes.|
| MPS | The maximum packet size a SOFT packet can have. The size includes the SOFT header.|
| File index     | Byte offset from which to start transferring a file|
> [name=nathaliepett] add more core terminology if we see fit...
> [name=vyas_g] The Connection ID is selected by the server and therefore can be either sequentially generated and assigned or it can be random. Any thoughts?
> [name=max] File Offset vs. File Index. Later we just mention Offset
> [name=max] Since we talk about SOFT packet we should use maximum packet size

## Objectives

> [name=nathaliepett] The following is taken from our spec directly, it is basically just the summary of the requirements from the assignment instructions, but I think it nicely sums up the objectives. Open to adjust it though (:

The Robust File Transfer (RFT) protocol was developed based on a set of instructions given as part of this assignment [assignment-1]. The protocol addresses a client-server scenario in which one or multiple files can be retrieved by a client from a server. One of the main requirements was to develop a protocol that MUST be built directly on top of UDP that MUST NOT be using another protocol on top of itself.
Furthermore, the protocol MUST be able to recover from connection drops and support connection migration. The main design goal of the protocol is that it MUST be reliable. It MUST also support flow control and minimal congestion control. To verify received files the protocol MUST support checksums. Some further assumptions are made to facilitate the design. Authentication, integrity protection, or encryption need not be addressed by the protocol. This opens up the protocol for some securitiy vulnerabilities discussed in the respective section.

> [name=benedikt] add individual goals too

- Protocol must be simple
    - easy to implement
- must support transfers upto TODO GB
- must support filenames upto TODO
- server may support up to TODO simultaneous connections
- ... TODO

# Layer Model

~~~ ascii-art
+-----------------------------------------------+
|                   SOFT Packet                 |
+-----------------------------------------------+
|                  UDP Datagram                 |
+ ----------------------------------------------+
|                   IPv4 Packet                 |
+-----------------------------------------------+
:                       ...                     :
~~~

Figure: Layer Model

# Timeouts

There are two types of timeout values: one is the retransmission timeout, determining the time to wait until a packet is retransmitted if an expected packet (either a data packet or an acknowledgment) is not received, the other one is the connection timeout, determining when the connection state is cleaned up, if expected packets are not received even after retransmission.

For the retransmission timeout, a value of two round trip times is selected.
> [name=nathaliepett] Maybe we should be more specific here on how we determine / how to calculate two RTTs?

> [name=max] I would suggest the moving average computation Group 10 mentioned in their spec.



For the connection timeout, a value of 60 seconds is chosen. If there is no packet received until timeout the connection SHOULD be closed.
> [name=nathaliepett] @M1YVKlXR40RzD6Ep I think you mentioned some reasoning for this concerning routers and their timeout being 60s, so maybe we should even pick a value slightly lower?

# Errors

| Error               | Code | Description                                        | Sent By         |
| ------------------- | ---- | -------------------------------------------------- | --------------- |
| STOP                | 0    | Graceful stop in the middle of a transfer          | Client          |
| UNKNOWN             | 1    | If no other error fits<br/> e.g. technical errors   | Client & Server |
| FILE_NOT_FOUND      | 2    | If requested file was not found by the server      | Server          |
| ACCESS_DENIED       | 3    | If file cannot be accessed by the server           | Server          |
| CHECKSUM_NOT_READY  | 4    | If server is not done generating the checksum      | Server          |
| INVALID_OFFSET      | 5    | If offset is larger than the file size             | Server          |
| UNSUPPORTED_VERSION | 6    | If protocol version is not supported by the server | Server          |
| FILE_CHANGED        | 7    | If file changed in the middle of a transfer        | Server          |
Figure: Errors

> [name=max] Special offset case: Client sends size of file as offset. This would be out of bound if offset starts at 0. Otherwise we could allow this special case. E.g. file size 100 bytes and offset 100 so that the client can "verify" it has a complete file.
> [name=max] Prof. Ott told me during the first assignment presentation that we can assume that the file is not changed during "error-free" file transfer. Otherwise a thread must check for writing changes in the implementation.

# Connection Establishment

This protocol's connection consists of the following parts:

* Connection Initiation
* File Transfer

The connection initiation starts with the following:
The client first sends a REQ packet to the server endpoint.
The server answers this with a ACC, which includes the checksum as well as the file size. Note, that with the transmission of the ACC packet, the server also sends the connection ID (CID). The client acknowledges the connection ID by sending the packet FileSend.

Connections are terminated implicitely by the server and client. The termination can be caused by errors, timeouts or after a file has been send successfully.

## Handshake Example

~~~ ascii-art
Client                      Server
   |                              |
   | -----------REQ-------------> |
   |  |  Protocol Version=1    |  |
   |  |     Packet Type=0      |  |
   |  |    Max Packet Size     |  |
   |  |        Offset          |  |
   |  |       File Name        |  |
   |                              |
   | <----------ACC-------------- |
   |  |  Protocol Version=1    |  |
   |  |     Packet Type=1      |  |
   |  |     Connection ID      |  |
   |  |       File Size        |  |
   |  |        Checksum        |  |
   |                              |
   | -----------ACK-------------> |  
   |  |  Protocol Version=1    |  |
   |  |     Packet Type=3      |  |
   |  |     Receive Window     |  |
   |  |     Connection ID      |  |
   |  | Next Sequence Number=0 |  |
   |                              |
   | <------File Transfer-------> | The file transfer is further detailed
   |                              | in the following section
   |                              |
~~~
Figure: Connection Establishment


# File Transfer

After receiving the acknowledgement of its ACC packet the server directly starts sending the first bytes of the requested file to the client.
Within the header of the first file, the client MUST gain following information:
1. Sequence number of the file data packet
2. Packet type
3. Payload of the actual sent file byte

With the sequence number the client is able to sort incoming packets correctly so that file the bytes are appended in correct order to the file. The client cumulatively acknowledges the file data packets.
In the acknowledgement packet the client sets the ack number to the next sequence number of the packet that is sent by the server in the next batch of packets. The packet type indicates if more file data packets will come.

~~~ ascii-art
Client                       Server
   |                              |
   | <--------Handshake---------> | The handshake is further detailed
   |                              | in the previous section
   |                              |
   | <-----------DATA------------ |
   |  |  Protocol Version=1    |  |
   |  |     Packet Type=2      |  |
   |  |     Connection ID      |  |
   |  |    Sequence Number=0   |  |
   |  |          Data          |  |
   |                              |
   | ------------ACK------------> |
   |  |  Protocol Version=1    |  |
   |  |     Packet Type=3      |  |
   |  |     Receive Window     |  |
   |  |     Connection ID      |  |
   |  | Next Sequence Number=1 |  |
   |                              |
   | <-----------DATA------------ |
   |  |  Protocol Version=1    |  |
   |  |     Packet Type=2      |  |
   |  |     Connection ID      |  |
   |  |    Sequence Number=1   |  |
   |  |          Data          |  |
   |                              |
   | ------------ACK------------> |
   |  |  Protocol Version=1    |  |
   |  |     Packet Type=3      |  |
   |  |     Receive Window     |  |
   |  |     Connection ID      |  |
   |  | Next Sequence Number=2 |  |
   |             ...              |
   |                              |
~~~
Figure: File Transfer

## Migration
It is important to note the difference when talking about connection migration and connection resumption. This protocol talks about "connection resumption" when the server does not have the state "connected" anymore.
Connection migration is referred to when both client and server have the state "connected" - which implies that the connection session is still existent between both endpoints.

## Example

~~~ ascii-art
Client                              Server
   |                                  |
   | --------------ACK--------------> |
   |  Source IP=116.0.0.1             |
   |  Destination IP=156.0.0.10       | 
   |  Source UDP Port=36004           |
   |  Destination UDP Port=9840       | 
   |  Connection ID=16284676044       |
   |                                  | ! Client IP changes 
   |                                  | from 116.0.0.1 to 116.0.0.2
   |                                  | and UDP Port changes
   |                                  | from 36004 to 36005
   |                                  |
   | X-------------DATA-------------- | Server sents to old IP/Port
   |  Source IP=156.0.0.10            |
   |  Destination IP=116.0.0.1        | *outdated
   |  Source UDP Port=9840            |
   |  Destination UDP Port=36004      | *outdated
   |  SOFT Connection ID=16284676044  |
   |                                  |
   |                                  | ! Client resend timeout
   |                                  |
   | --------------ACK--------------> | 
   |  Source IP=116.0.0.2             | *new 
   |  Destination IP=156.0.0.10       | 
   |  Source UDP Port=36005           | *new
   |  Destination UDP Port=9840       | 
   |  SOFT Connection ID=16284676044  | Server recognizes the
   |                                  | Connection ID and updates
   |                                  | the IP and Port
   |                                  |
   | <--------------DATA------------- | Server sends to updates IP/Port
   |  Source IP=156.0.0.10            |
   |  Destination IP=116.0.0.2        | *updated 
   |  Source UDP Port=9840            |
   |  Destination UDP Port=36005      | *updated 
   |  SOFT Connection ID=16284676044  |  
   |                                  |
~~~
Figure: Migration Example

## Resumption
When connection needs to be resumed, both client and server have lost their "connected" status - which means that the their connection session does not exist anymore.
Therefore, the connection establishement has do be done again with the three-way-handshake. With this, the client will receive a new connection ID.
The ACC packet - which server sends to the client - includes the server's checksum. This checksum will be compared by the client with it's own previously received stored checksum.
Two scenarios may happen:
1. Client's previously stored checksum and the newly received checksum are identical: The file transfer will be resumed.
2. Client's previously stored checksum and the new received checksumare not identical: This implies, that the file changed server-side and the next data streams from the server will be inconsistent to the client's received data bytes. The client will therefore send another REQ with OFFSET set to 0 - which tells the server that the file needs to be sent starting from the first byte.
> [name=nathaliepett] just as a reminder for us: on resumption (and migration) we wanted to compare the file checksums client side

> [name=vyas_g] Resumption of a file transfer is treated as a new request for a file from a specific file offset. Needs a diagram to explain flow.

> [name=max] @vyasg indeed. Also both possibilities with the file checksum. We should draw the diagram(s) after the diagram for the "classic" file request/transfer is made.

# Flow & Congestion Control

> [name=benedikt] if the client starts with an ACK 0, no receive window field has to be included in the REQ packet

> [name=nathaliepett] I think that's a good idea for two reasons: the one that you mentioned, and also, if we are generally working with forward acknowledgments (i.e. the segment number in the ack is always the next expected one) it only makes sense to have an ack before the first package is sent, so we are consistent (otherwise the first data packet would never have an ack, if everything goes well)

## Congestion Control

> [name=nathaliepett] Again this is taken from our spec, open for discussion / adjustments.

The RFT protocol MUST include minimal congestion control. To this end we propose a simplified version of TCP Reno [@RFC5681] including a slow start and a congestion avoidance phase. The principle of additive increase / multiplicative decrease (AIMD) is adhered to.

The server maintains a congestion window (cwnd) in addition to the receive window communicated by the client. Whichever one is smaller in size determines how much data is actually sent:

~~~
MaxWindow = min{ReceiveWindow, CongestionWindow}
EffectiveWindow = MaxWindow - (LastPacketSent - LastPacketAcked)
~~~

with the *EffectiveWindow* indicating how much data can be sent.
The *MaxWindow* is the maximum number of unacknowledged data allowed in circulation.
The *ReceiveWindow* is sent by the opposing endpoint in every *ACK* packet and is comprised of the receive buffer size.

If the *EffectiveWindow* is greater than 0 more data can be transmitted.

The initial congestion window size is set to one maximum packet size.
During the slow start phase the congestion window is increased by one maximum segment size per acknowledged segment. Eventually, when three duplicate acknowledgments are received, the threshold for congestion avoidance is set to half the size of the last congestion window and the congestion window is adjusted to this size as well.

~~~
Let w(t) bet the congestion window size at time t:

w(0) = alpha
w(t+1) = w(t) + alpha    if no congestion is detected
w(t+1) = w(t) * beta     if congestion is detected

~~~

The additive increase factor *alpha* is chosen to be one maximum packet size.
The multiplicative decrease factor *beta* is chosen as 1/2 which results in halfing the congestion window if congestion is detected.

Then, the congestion avoidance phase starts. During congestion avoidance, the window size is only increased by (1/cwnd) per acknowledged segment. The behavior in case of three duplicate acknowledgements is repeated. If at any time a timeout occurs, the threshold for congestion avoidance is set to half the current congestion window size, the congestion window is set to 1 maximum segment size and a new slow start phase that continues until the congestion avoidance threshold is started.

There are drawbacks of sending one file per connection when it comes to congestion control, as each connection would per se start with a new slow start phase. To mitigate this effect and avoid slow start phases for each new connection the client remembers its last receive window of the previous connection in case there are still more files contained in the current request, this way it can be reused for connections related to one request.
> [name=Thomas Midek] Remembers for how long ? 10 min ?

The initial *RTT* is choosen to be 3 seconds. The *RTT* is then updated using a moving average:

{align="center"}
~~~
RTT = gamma * oldRTT + (1-gamma) * newRTTSample
~~~

with *gamma* denoting a constant weighting factor.

The new *RTT* samples are obtained as the time measured between a packets transmission and the reception of its acknowledgment.

### Congestion Window Caching
The SOFT protocol is designed for transfering single files.
To transfer multiple small files, a new connection must be initialized for each one.
This can drastically reduces the throughput, when the congestion window is reset for every new connection.
That is why we recommend the server to use congestion window caching i.e. despite closing the connection, the server remembers the congestion window that is associated with the IP and UDP port (not connection ID).
Therefor we also recommend the client to reuse the UDP port for multiple file transfers.
As cache timeout we recommend 10 times the RTT.

## Flow Control

# Flow Example

TODO example connection and transfer flow


# Packet Types

All packets share the protocol version, currently 0x01, and the operation id fields.  The operation id is used to distinguish the various different types of packets that RFT knows.
The Maximum segment/packet size always refers to the whole udp payload (e.g. our payload + header).

~~~ ascii-art
 0               1               2               3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|   Version=1   |  Packet Type  |                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+                               |
|                                    Packet Type Dependent      |
:                                                               :
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
~~~
Figure: General Packet encoding



| Type | Code | Description                 | Sent By         |
| ---- | ---- | --------------------------- | --------------- |
| REQ  | 0    | initial request of a file   | Client          |
| ACC  | 1    | accepted file transfer      | Server          |
| DATA | 2    | containing file data        | Server          |
| ACK  | 3    | acknowledge received data   | Client          |
| ERR  | 4    | abort connection with error | Client & Server |
Table: Packet Types



| Field                | Size       | Encoding         | Comment                                                |
| -------------------- | ---------- | ---------------- | ------------------------------------------------------ |
| Version              | 1 Byte     | unsigned integer | protocol version is always 1 for current specification |
| Packet Type          | 1 Byte     | unsigned integer (Big-Endian) | one of the defined packet type codes                   |
| Max Packet Size      | 2 Byte     | unsigned integer (Big-Endian) | maximum SOFT packet size supported by the client       |
| Receive Window       | 4 Byte     | unsigned integer (Big-Endian) |                                                        |
| File Name            | ≤ 484 Byte | UTF-8            | length is specified by datagram size                   |
| File Size            | 8 Byte     | unsigned integer (Big-Endian) |                                                        |
| Connection ID        | 4 Byte     | unsigned integer (Big-Endian) |                                                        |
| Checksum             | 32 Byte    | SHA-256          |                                                        |
| Offset               | 8 Byte     | unsigned integer (Big-Endian) | transfer starts at this byte index of file             |
| Sequence Number      | 8 Byte     | unsigned integer (Big-Endian) |                                                        |
| Next Sequence Number | 8 Byte     | unsigned integer (Big-Endian) |                                                        |
| Data                 | variable   | binary           | length is specified by datagram size                   |
| Error Code           | 1 Byte     | unsigned integer | one of the defined error codes                         |
Table: Fields



## File Request Packet (REQ)

- 1B protocol version
- 1B packet type: 0x00
- 2B max segment size supported by client
- 8B offset
- 484B filename

The filename size is based on the minimal IPv4 packet size network hosts must support. [@RFC791] sets this to 576 bytes. Considering the IPv4 header (40 bytes), the possibility of IPv4 options (20 bytes) and the UDP datagram (20 bytes) the filename size can maximally be as followed:

**Note:** SOFT REQ HEADER - everything that is **not the file name**
(576 - 40 (IPv4 header) - 20 (IPv4 Options) - 20 (UDP Datagram) - 12 (SOFT REQ Header)) byte = 484 byte


~~~ ascii-art
 0               1               2               3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|   Version=1   | Packet Type=0 |      Max SOFT Packet Size     |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                            Offset                             |
:                                                               :
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                          File Name                            |
:                                                               :
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
~~~

## Accept File Transfer Packet (ACC)

- 1B protocol version
- 1B packet type: 0x01
- 4B connection ID
- 8B file size in bytes
- 32B SHA 256 checksum

~~~ ascii-art
 0               1               2               3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|   Version=1   | Packet Type=1 |           padding             |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Connection ID                         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                                                               |
|                       File Size (in Byte)                     |
|                                                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                                                               |
|                                                               |
|                                                               |
|                                                               |
|                                                               |
|                                                               |
|                   Checksum (SHA-256, 32 Byte)                 |
|                                                               |
|                                                               |
|                                                               |
|                                                               |
|                                                               |
|                                                               |
|                                                               |
|                                                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
~~~

Note that the padding is unused space only used for alignment.


## Data Packet (DATA)

- 1B protocol version
- 1B packet type: 0x02
- 4B connection ID
- 8B sequence number
- variable size data/payload

~~~ ascii-art
 0               1               2               3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|   Version=1   | Packet Type=2 |            padding            |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                        Connection ID                          |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                                                               |
|                       Sequence Number                         |
|                                                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                     Data (variable size)                      |
:                                                               :
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
~~~

## Acknowledgement Packet (ACK)

- 1B protocol version
- 1B packet type: 0x03
- 2B receive window
- 4B connection ID
- 8B next sequence number

~~~ ascii-art
 0               1               2               3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|   Version=1   | Packet Type=3 |      Receive Window           |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                        Connection ID                          |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                                                               |
|                    Next Sequence Number                       |
|                                                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
~~~


## Error Packet (ERR)

- 1B protocol version
- 1B packet type: 0x04
- 4B connection id
- 1B error code

~~~ ascii-art
 0               1               2               3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|   Version=1   | Packet Type=4 |   Error Code   |   padding    |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                        Connection ID                          |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
~~~

Note that the padding is unused space only used for alignment.



# IANA Considerations

This memo includes no request to IANA.



# Future Work

## Considerations on IP Layer Fragmentation

> [name=nathaliepett] TODO: @Group6 add section on IP Layer Fragmentation form your spec

## IPv6 Support
TODO
> [name=max] Filename size other computation
# Security

Security considerations have not been a paramount concern during the initial development of this protocol. Therefore, the following potential threats have not yet been addressed. This section might be expanded in future revisions of this protocol.

## Availability

### Denial of Service

Denial of service attacks to render the server unresponsive are possible in the current version of this protocol. To prevent the server from crashing, because of full memory, it might reject new connections when a certain threshold is reached. This however, does not prevent attackers from filling the connection queue with requests so a genuine client cannot get their request accepted.

## Integrity

### Session Hijacking

As encryption was not a requirement for the protocol and no other protocols are used below or on top of this protocol which would provide it for us, there is little to be done to prevent session hijacking if the connection id becomes compromised.

## Confidentiality

### Encryption
TODO

## Third Threat

TODO
> [name=nathaliepett] TODO: @jhBvPbYsSYaoRKT5crw-NQ You might want to insert the other threat you mentioned. It escapes me currently what exactly it was.

{backmatter}