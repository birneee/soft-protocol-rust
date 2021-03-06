%%%
title = "SOFT Protocol (Draft)"
abbrev = "SOFT Protocol"
ipr= "trust200902"
area = "Transport"
workgroup = "Group 0, 6, 10"
submissiontype = "IETF"
keyword = ["FTP", "TCP"]
date = 2021-08-12T00:00:00Z

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
[author.address]
email = "thomas.midek@tum.de"

[[author]]
initials = "V."
surname = "Giridharan⁩"
fullname = "Vyas Giridharan⁩"
organization = "Technische Universität München"
[author.address]
email = "giridhar@in.tum.de"

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
[author.address]
email = "nathalie.pett@tum.de"

[[author]]
initials = "F."
surname = "Gust⁩"
fullname = "Felix Gust⁩"
organization = "Technische Universität München"

[[author]]
initials = "MN."
surname = "Naqvi"
fullname = "Musfira Naqvi"
organization = "Technische Universität München"

[[author]]
initials = "M."
surname = "Wiesholler⁩"
fullname = "Maximilian Wiesholler⁩"
organization = "Technische Universität München"
[author.address]
email = "maximilian.wiesholler@tum.de"

%%%

.# Abstract
The SOFT (Simple One File Transfer) protocol has the goal of enabling robust file transfers over the network encapsulated in UDP datagrams.
The protocol transports one file per connection.

{mainmatter}

{#introduction}
# Introduction

{#requirements-language}
## Requirements Language
The keywords **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL**, when they appear in this document, are to be interpreted as described in [@RFC2119].

{#terminology}
## Terminology
| Term        | Description                                                                                                               |
| ----------- | ------------------------------------------------------------------------------------------------------------------------- |
| Client      | Entity requesting one or more files from the server                                                                       |
| Server      | Entity providing one or more files to the client                                                                          |
| Connection  | A connection is identified by a unique connection ID and comprises all interaction necessary to transfer a single file |
| Packet      | A SOFT packet is comprised of a header and some payload. There are different types of packets for different purposes  |
| MPS         | The maximum packet size a SOFT Data packet can have. The size includes the SOFT header                                        |
| File Offset | Byte offset from which to start transferring a file                                                                       |
| Migration   | The client's IP or port number changes during file download but the connection is not interrupted                                                                                                                       |
| Resumption  | A previous partial file download is resumed via a new connection                                                                                                                         |
| RTT  | Round Trip Time. The time between sending a datagram and receiving the corresponding response                                                                                                                           |
Table: Terminology

{#objectives}
## Objectives

The SOFT protocol was developed based on a set of instructions given as part of this assignment [@assignment-1]. The protocol addresses a client-server scenario in which a single or multiple files can be retrieved by a client from a server. One of the main requirements was to develop a protocol that MUST be built directly on top of UDP that MUST NOT be using another protocol on top of itself.
Furthermore, the protocol MUST be able to recover from connection drops and support connection migrations. The main design goal of the protocol is that it MUST be reliable. It MUST also support flow control and minimal congestion control. To verify received files the protocol MUST support checksums. Some further assumptions are made to facilitate the design. Authentication, integrity protection, or encryption need not be addressed by the protocol. This opens up the protocol for some security vulnerabilities discussed in the respective section.

In addition, the protocol MUST be transportable over every IPv4 network.
The protocol SHALL be easy to understand and implement.
The protocol SHALL be able to efficiently transfer small and large files.
File sizes from 1 byte up to 18 exabytes SHALL be supported.
The server MAY support multiple simultaneous connections to clients.
The protocol MUST support file names from 1 up to 255 bytes.

{#state-information}
# State Information
For the correct operation of SOFT some state needs to be maintained on both the client and the server side. In particular this refers to various timeout values. They are described in the following section along with more information on how the round trip times (RTTs) needed for these timeouts are determined.

{#timeout-values}
## Timeout Values

There are various types of timeout values:


| Timeout                            | Value             | Description                                                                                                                                                                                                                                                                                                                                            |
| ---------------------------------- | ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| ACK Packet Retransmission Timeout  | max(3 RTT, 100ms) | Determines the time to wait until an ACK packet is retransmitted if an expected DATA packet is not received. The ACK is then retransmitted every 3 RTTs. This purpose of this timeout is to notify the server about migrations. The 100ms threshold prevents sending ACKs too often, because it could be interpreted as congestion on the server side. |
| DATA Packet Retransmission Timeout | 2 RTTs*           | Determines the time to wait until a DATA packet is retransmitted if an expected ACK packet is not received. The DATA packet is then retransmitted every 2 RTTs. *The server MIGHT double the retransmisssion timeout for consecutive timeouts.                                                                                                          |
| Connection Timeout                 | max(20 RTT, 5s)   | Determines when the connection state is cleaned up, if expected packets are not received even after retransmission. The 5 second threshold helps to not close low-RTT connection on minor delays.                                                                                                                                                      |
| Path Cache Timeout                 | max(20 RTT, 5s)   | Determines when the entry in the path cache is cleaned up (see (#path-caching)). The 5 second threshold helps to ensure that low-RTT path information is not cleaned up too early.                                                                                                                                                                          |
| Packet Loss Timeout                | 2 RTTs            | Multiple duplicate ACK packets with same sequence number are only interpreted as one packet loss in that time frame.                                                                                                                                                                                                                                    |
Table: Timeouts

The initial RTT is 3 seconds.
More on how the RTT is calculated can be found in the respective section (#rtt-measurements).

{#rtt-measurements}
## Roundtrip Time Measurement
The client and server must estimate the RTT to calculate the timeouts (see (#timeout-values)).

### Server RTT Measurement

The initial *RTT* is chosen to be 3 seconds. The *RTT* is then updated using a moving average:

{align="center"}
~~~
RTT = gamma * oldRTT + (1-gamma) * newRTTSample
~~~


with *gamma* denoting a constant weighting factor.


The new *RTT* samples are obtained as the time measured between a DATA packet's transmission and the reception of its corresponding ACK packet.
Duplicate ACKs are ignored for the RTT measurement.
The server can also use the time between the transmission of the ACC packet and the reception of the ACK 0 packet as an RTT sample.

We recommend the gamma value to be 0.5 and to create one sample per RTT.

### Client RTT Measurement

The client uses the time between the transmission of the first ACK packet (with sequence number 0) and the reception of the first DATA packet (with sequence number 0) as the RTT.


In the current version of the protocol the client is not required to update the RTT during a connection but may do so if established helpful.
Therefore, the RTT is only used as a rough estimate for ACK retransmission.
This might be addressed in future versions because it leads to problems when the network conditions change, especially with connection migration.


{#protocol-operation}
# Protocol Operation
This section gives detailed insights into how the protocol operates. SOFT is a transfer protocol operating on top of UDP and IPv4, as depicted in the following diagram. Considerations about making it IPv6 ready can be found in (#future-work).
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

{#protocol-phases-and-events}
## Protocol Phases and Special Events

A SOFT protocol connection consists of a connection initiation phase and a file transfer phase. These phases are described in more detail in the upcoming sections. During a transfer a client might migrate or resume a previous connection. These special events are also addressed in this section. Connections are terminated implicitly by the server and client. The termination can be caused by errors, timeouts or after a file has been sent successfully.

{#connection-initiation}
### Connection Initiation (including example)
The client is always the initiator of the connection.
Therefore, the client has to know the IP and the UDP port of the server.
We recommend the server implementation to use 9840 as the default port.


Connections are initialized by a three-way handshake.
The connection initiation works as follows:
The client first sends a REQ packet to the server endpoint.
The server answers this with an ACC packet, which includes the checksum as well as the file size. Note, that with the transmission of the ACC packet, the server also sends the connection ID (CID). The client acknowledges the connection ID by sending the packet ACK 0.

The connection ID should be generated randomly by the server.

Refer to the following example handshake for deeper understanding.

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

{#file-transfer-phase}
### File Transfer Phase (including example)

After the handshake (i.e. the ACK 0) the server directly starts sending the first bytes of the requested file to the client as part of a DATA packet (#data-packet).
With the sequence number the client is able to sort incoming packets correctly so that the bytes are appended in correct order to the file.
By knowing the file size both endpoints are able to determine if there are more data packets to transfer.

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

{#migration}
### Connection Migration

The SOFT protocol supports connection migration of the client.
This means the source IP or UDP port of the client can change during a connection, without interruption of the file transfer. Specifically, on migration, there are no checks whether the transferred file has changed on the server as the connection is upheld during migration.
This draft talks about connection migration, when both the client and the server still have all state information relating to the current connection stored. It is different to connection resumption, for which this is not the case (see (#connection-resumption)).


When the server notices a migration of the client, it should reset the congestion window to the initial value of 1 MPS.
An example, for when connection migration might be especially useful, is when mobile clients want to switch to a different network interface.

When the connection is migrated the congestion information and the RTT have to be reset to the initial values.
The server MIGHT use already cached values from the Path Cache (see #path-caching).

The following diagram depicts what happens during connection migration:

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

It is assumed that only the client has the ability to migrate to another address. Server migration is not supported by this protocol.

{#connection-resumption}
### Connection Resumption
This protocol talks about connection resumption, when the client wants to resume a (partial) file download after the state associated with a connection has already been discarded.
To do this, another three-way handshake for connection establishment has to be performed (see (#connection-initiation)).
The difference between an initial handshake and a resumption handshake is, that the offset communicated is not 0.
The client sets the offset in the REQ packet to the byte index of the file at which it wants to proceed with the transfer.
The ACC packet includes a new connection ID and the server's file checksum.
This checksum will be compared by the client to its own, previously received stored checksum.


Three scenarios may happen:

1. The client's previously stored checksum and the newly received checksum are identical: The file transfer will be resumed.
2. The client's previously stored checksum and the new received checksum are not identical: This implies, that the file has changed server-side and the next data streams from the server will be inconsistent to the clients received data bytes. The client will therefore send another REQ with OFFSET set to 0 - which tells the server that the file needs to be sent starting from the first byte.
3. If the client receives an **InvalidOffset** error while trying to resume, this means that the server's file has reduced in size. The client MIGHT initiate a new SOFT connection with offset 0, to receive the new version.

{#acknowledgments}
## Acknowledgments
Only DATA packets are acknowledged by the client.
SOFT uses positive cumulative forward acknowledgements.
The client should acknowledge each received DATA packet immediately.
If the client receives a DATA packet with a higher sequence number than expected, it will immediately send an ACK packet with the sequence number of the next DATA packet it wants to receive.
These duplicate ACK packets are used by the server to detect packet loss and congestion (see (#congestion-control)).


Because the server might receive many duplicate ACK packets for the same sequence number, the server should not interpret this as multiple packet losses.
The server has to use a Packet Loss Timer (see (#timeout-values)). Within this time multiple ACK packets with the same sequence number do lead to retransmission and halving of the congestion window only once.


Currently the protocol uses the "go-back-n" strategy in case of packet loss or packets received out of order. This means that if the client receives a DATA packet it is not expecting as the next packet, this packet is discarded and an acknowledgment indicating the sequence number of the next packet expected is sent.
Therefore the client does not accept reordered packets.

{#retransmission}
## Retransmission

If the client does not receive any *DATA* packet for some time (see (#timeout-values)), it will resend its last *ACK* packet.
If the server does not receive any *ACK* packet for some time (see (#timeout-values)), it will resend its last *DATA* packet.
Not receiving any packet for some time might be an indication for migration (see (#migration)).

{#errors}
## Errors
| Error                | Code | Description                                            | Sent By         |
| -------------------- | ---- | ------------------------------------------------------ | --------------- |
| STOP                 | 0    | Graceful stop in the middle of a transfer              | Client          |
| INTERNAL             | 1    | If no other error fits<br/> e.g. technical errors      | Client & Server |
| FILE\_NOT\_FOUND     | 2    | If requested file was not found by the server          | Server          |
| BAD\_PACKET          | 3    | If the received packet contains invalid fields         | Client & Server |
| CHECKSUM\_NOT\_READY | 4    | If server is not done generating the checksum          | Server          |
| INVALID\_OFFSET      | 5    | If offset is larger than the file size                 | Server          |
| UNSUPPORTED\_VERSION | 6    | If protocol version is not supported by the server     | Server          |
| FILE\_CHANGED        | 7    | If file changed in the middle of a transfer/connection | Server          |
Table: Errors

Until the handshake has finished and the client has successfully obtained a connection ID, the client must ignore the connection ID field of incoming error packets.
Every error implicitly closes a connection.

Because Error packets are not acknowledged they might get lost.
For the receiver, This would lead to a normal connection timeout.
An implementation MAY send the same Error packet multiple times to increase the likelihood that the other side will receive it.

If client or server receive an unkown or unspecified error code they should nevertheless close the connection, because the list of specified error codes might be extended in future versions.

Instead of sending an Error, implementations MAY ignore certain invalid packets.

{#flow-and-congestion-control}
# Flow Control and Congestion Control

{#flow-control}
## Flow Control
Flow control is achieved by letting the client communicate its current ReceiveWindow. It is sent by the client endpoint in every ACK packet and is comprised of the receive buffer size.
The ReceiveWindow is the number of packets that can be received by the client.
The maximum packet size supported by the client's buffer is the sent MPS in the REQ packet.

{#congestion-control}
## Congestion Control
The SOFT protocol MUST include minimal congestion control. To this end we propose a simplified version of TCP Reno [@RFC5681] including a slow start and a congestion avoidance phase. The principle of additive increase / multiplicative decrease (AIMD) is adhered to.

The server maintains a congestion window (cwnd) in addition to the receive window communicated by the client. Whichever one is smaller in size determines how much data is actually sent:

~~~
MaxWindow = min{ReceiveWindow, CongestionWindow}
EffectiveWindow = MaxWindow - (LastPacketSent - LastPacketAcked)
~~~

with the *EffectiveWindow* indicating how much data can be sent.
The *MaxWindow* is the maximum number of unacknowledged DATA packets allowed in circulation.
If the *EffectiveWindow* is greater than 0 more data may be transmitted.


The initial congestion window size is set to one maximum packet size (MPS).
During the slow start phase the congestion window is increased by one MPS per received ACK packet. Eventually, when two duplicate ACK packets are received, the threshold for congestion avoidance is set to half the size of the last congestion window and the congestion window is adjusted to this size as well.

~~~
Let w(t) be the congestion window size at time t:

w(0) = alpha
w(t+1) = w(t) + alpha    if no congestion is detected
w(t+1) = w(t) * beta     if congestion is detected
~~~

The additive increase factor *alpha* is chosen to be one MPS.
The multiplicative decrease factor *beta* is chosen as 1/2 which results in halving the congestion window if congestion is detected.


Then, the congestion avoidance phase starts. During congestion avoidance, the window size is only increased by (1/cwnd) per acknowledged packet. The behavior in case of two duplicate acknowledgments is repeated. If at any time a retransmission timeout occurs, the threshold for congestion avoidance is set to half the current congestion window size, the congestion window is set to 1 MPS and a new slow start phase that continues until the congestion avoidance threshold is started.

{#path-caching}
### Path Caching
A SOFT connection is designed to transfer single files only. In order to transfer multiple files, a new connection must be initialized for each one. There are drawbacks of this behavior when it comes to congestion control, as each connection would per se start with a new slow start phase resetting the congestion window. This can drastically reduce the throughput, especially with multiple, small files. To mitigate this effect and avoid slow start phases for each new, but related connection it is recommended to use server-side path caching, i.e. despite closing the connection, the server remembers the congestion information and the RTT that is associated with the IP and UDP port (not connection ID).
Therefore we also recommend the client to reuse the same UDP port for multiple file transfers.
For the cache timeout see (#timeout-values).

{#packet-types}
# Packet Types and Encoding
All packets share the protocol version, currently 0x01, and the packet type fields.  The packet type is a numerical value used to distinguish the various different types of packets that SOFT supports.
The MPS always refers to the whole UDP payload (i.e. the SOFT header and SOFT payload).

~~~ ascii-art
 0               1               2               3
 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|   Version=1   |  Packet Type  |                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+                               |
|                                    Packet Type Dependent      |
:                                                               :
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
~~~
Figure: General Packet encoding

The following table lists all currently supported packet types.

| Type | Code | Description                 | Sent By         |
| ---- | ---- | --------------------------- | --------------- |
| REQ  | 0    | initial request of a file   | Client          |
| ACC  | 1    | accepted file transfer      | Server          |
| DATA | 2    | containing file data        | Server          |
| ACK  | 3    | acknowledge received data   | Client          |
| ERR  | 4    | abort connection with error | Client & Server |
Table: Packet Types

All SOFT packets can be encapsulated in a minimal IPv4 packet, therefore transportability on networks can be guaranteed.
The only exception is the DATA packet, its size is limited by the minimal MPS from client and server (see (#data-packet)).
The client can probe the MPS supported by the network by choosing different MPS values in the REQ packet.

The following table lists all possible fields including their size and encoding. Refer to the comments for further explanations of the fields.

| Field                | Size                 | Encoding                      | Comment                                                                |
| -------------------- | -------------------- | ----------------------------- | ---------------------------------------------------------------------- |
| Version              | 1 byte               | unsigned integer              | Protocol version is always 1 for current specification                 |
| Packet Type          | 1 byte               | unsigned integer (Big-Endian) | One of the defined packet type codes                                   |
| Max Packet Size      | 2 byte               | unsigned integer (Big-Endian) | Maximum SOFT packet size supported by the client                       |
| Receive Window       | 2 byte               | unsigned integer (Big-Endian) | Number of Packets, the client is able to receive (Flow control)        |
| File Name            | variable <br/> > 0 byte <br/> <= 484 byte          | UTF-8                         | Length is specified by datagram size                                   |
| File Size            | 8 byte               | unsigned integer (Big-Endian) | The total file size in bytes                                                               |
| Connection ID        | 4 byte               | unsigned integer (Big-Endian) | Identifier for the Connection, chosen by the server                                                                        |
| Checksum             | 32 byte              | SHA-256                       | Checksum of the file content                                                                       |
| Offset               | 8 byte               | unsigned integer (Big-Endian) | Transfer starts at this byte index of file                             |
| Sequence Number      | 8 byte               | unsigned integer (Big-Endian) | Sequence number of the DATA packet                                                                       |
| Next Sequence Number | 8 byte               | unsigned integer (Big-Endian) | Expected sequence number by the client                                                                       |
| Data                 | variable <br/> > 0 bytes | binary                        | Length is limited by the maximum UDP payload, the MPS requested by the client and the supported MPS of the server |
| Error Code           | 1 byte               | unsigned integer              | One of the defined error codes                                         |
Table: Fields

{#req-packet}
## File Request Packet (REQ)

- 1 byte protocol version
- 1 byte packet type: 0
- 2 byte max segment size supported by client
- 8 byte offset
- variable length file name

The file name has to be at least 1 byte and maximum 484 byte.
The server can calculate the length of the file name via the UDP datagram size.


The maximum file name size is based on the minimal IPv4 packet size network hosts must support. [@RFC0791] sets this to 576 bytes. Considering the IPv4 header (40 bytes), the possibility of IPv4 options (20 bytes) and the UDP datagram (20 bytes) the maximum file name size can be calculated as follows:


Note: SOFT REQ HEADER - everything that is not the file name


~~~
(576 - 40 (IPv4 header) - 20 (IPv4 Options) - 20 (UDP Datagram Header) - 12 (SOFT REQ Header)) byte = 484 byte
~~~


Currently the protocol only supports IPv4.
For the consideration of IPv6 see (#ipv6-support).

The client has to choose the Maximum Packet Size (MPS) field in the REQ packet to avoid IP fragmentation.
Most networks have an MTU of 1280 or higher.
That is why we recommend a default MPS of 1200 byte.

~~~
(1280 (MTU) - 40 (IPv4 header) - 20 (IPv4 Options) - 20 (UDP Datagram Header) byte = 1200 byte
~~~


~~~ ascii-art
 0               1               2               3
 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|   Version=1   | Packet Type=0 |      Max Packet Size          |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                                                               |
|                            Offset                             |
|                                                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                          File Name                            |
:                                                               :
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
~~~
Figure: REQ packet

{#acc-packet}
## Accept File Transfer Packet (ACC)

- 1 byte protocol version
- 1 byte packet type: 1
- 4 byte connection ID
- 8 byte file size in bytes
- 32 byte SHA 256 checksum

~~~ ascii-art
 0               1               2               3
 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 
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
Figure: ACC packet

Note that the padding is unused space only utilized for alignment.
The padding should be set to 0 and should be ignored by the current version, because it might be used by future versions.

{#data-packet}
## Data Packet (DATA)

- 1 byte protocol version
- 1 byte packet type: 2
- 4 byte connection ID
- 8 byte sequence number
- variable size data payload

The data field has to be at least 1 byte.
The maximum data field size is limited by the maximum UDP payload, the MPS requested by the client, and the supported MPS of the server.

~~~
EffectiveMps = min{MaxUdpPayload, RequestedClientMps, SupportedServerMps}
~~~

The client can calculate the length of the data payload via the UDP datagram size.


~~~ ascii-art
 0               1               2               3
 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 
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
Figure: DATA packet

Note that the padding is unused space only utilized for alignment.

The padding should be set to 0 and should be ignored by the current version, because it might be used by future versions.

{#ack-packet}
## Acknowledgement Packet (ACK)

- 1 byte protocol version
- 1 byte packet type: 3
- 2 byte receive window
- 4 byte connection ID
- 8 byte next sequence number

~~~ ascii-art
 0               1               2               3
 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 
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
Figure: ACK packet

{#err-packet}
## Error Packet (ERR)

- 1 byte protocol version
- 1 byte packet type: 4
- 4 byte connection id
- 1 byte error code

~~~ ascii-art
 0               1               2               3
 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 0 1 2 3 4 5 6 7 
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|   Version=1   | Packet Type=4 |   Error Code   |   padding    |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                        Connection ID                          |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
~~~
Figure: ERR packet

Note that the padding is unused space only utilized for alignment.

The padding should be set to 0 and should be ignored by current version, because it might be used by future versions.

{#iana}
# IANA Considerations
This memo includes no request to IANA.

{#security-considerations}
# Security Considerations
Security considerations have not been a paramount concern during the initial development of this protocol. Therefore, the following potential threats have not yet been addressed by this protocol version. This section might be refined in future revisions of this protocol.

For future versions of this protocol comprehensive measures are necessary to address some of the major security concerns of this protocol. As the vulnerabilities mentioned in the following can be used to deny the main goal of this protocol, reliability, to the communication participants by denying them timely access to the correct, unmodified files, these security issues are of major concern for future versions.

{#availability}
## Availability
{#dos}
### Denial of Service
Denial of service attacks to render the server unresponsive are possible in the current version of this protocol. To prevent the server from crashing due to full memory, it might reject new connections when a certain threshold is reached. This, however, does not prevent attackers from filling the connection queue with requests so a genuine client cannot get their request accepted.

Also, there is a specific vulnerability to DoS attacks as the server, depending on the specific implementation, may calculate file checksums on the fly which is very compute intensive. This behavior can be exploited in a REQ attack to deplete a servers compute or IO resources.

Therefore the server SHOULD cache checksums and calculate them in a separate thread, without blocking other incoming REQ packets. The server can immediately respond with a ChecksumNotReady Error without creating any connection state.

For normal requests (when the checksum is ready) the server has to create a connection state at the first REQ packet, that is why we propose Request Cookies for future versions of the protocol (see (#request-cookies)).

{#integrity}
## Integrity
The SOFT protocol does not provide any specific means to assure integrity. Although the use of checksums ensures data integrity in an assumed non-compromised environment, the lack of integrity protection makes various fields vulnerable against an attacker.

| Field           | Possible Attack                                                                                                                       |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| Version         | Attacker might change protocol version to a less secure version or version with lower performance                                     |
| Packet Type     | Attacker might change the packet type to an error packet interrupt connection to deny service             |
| MPS             | Attacker might reduce quality of service                                                                                              |
| File Name       | Attacker might tamper the file name to a larger file, which leads to a longer file transfer |
| File Size       | Attacker might tamper the file size leading to a premature termination of a file transfer                                             |
| Offset          | Attacker might tamper the offset leading to an invalid file after file checksum computation                                           |
| Checksum        | Attacker might change checksum making the file checksum mechanism of SOFT untrustworthy                                              |
| Data        | Attacker might unnoticeably modify or replace content with wrong or malicious data                                               |
Table: Confidential Fields


{#confidentiality}
## Confidentiality
The SOFT protocol does not provide any measures against eavesdropping.

Besides the File Name and the Data, other protocol fields also should be protected with regards to confidentiality, because such metadata may also allow attackers to infer information about the transferred content.

Some fields that should be confidentiality protected:


| Field           | Possible Attack                                                                                 |
| --------------- | ----------------------------------------------------------------------------------------------- |
| File Name       | Attacker might infer content                                                                    |
| File Size       | Attacker might infer content, by comparing with known files                                     |
| Connection ID   | Attacker might be able to track user on long-lasting transfers, even across multiple migrations |
| Checksum        | Attacker might infer content, by comparing with known files                                     |
| Offset          | Attacker might infer file size                                                                  |
| Sequence Number | Attacker might infer file size                                                                  |
| Data            | Attacker can directly read content                                                              |
Table: Confidential Fields

{#encryption}
### Encryption
The SOFT protocol does not provide any method of ensuring the confidentiality of a packet payload or -header.
SOFT packets could be encapsulated in another encrypted transport protocol.
Future versions of the SOFT protocol might specify a standard way of doing this.

{#authentication}
## Authentication
In the handshake of the SOFT protocol neither the server nor the client is authenticated.
To ensure originality of the transferred file, authentication of the server is necessary. The current SOFT protocol is vulnerable for man-in-the-middle attacks since the client has no option to validate the server's identity.
In order to manage access control on the files, client authentication is necessary. Since the current SOFT protocol lacks client authentication all clients have the same access rights for all provided files.
To guarantee authentication, an entity must send a certificate in the handshake so that the receiver can validate the identity of its communication partner.

Some fields in the ACK packet that should be authenticated:


| ACK Field                             | Possible Attack                                                                                                                |
| ------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| Connection ID                         | Attacker might route traffic to other location with custom ACK packet, and deny service for the client                         |
| Next Sequence Number & Receive Window | Attacker can use ACK flooding to ramp up the congestion window, which can be used to DoS network infrastructure and the client |
Table: Authenticated ACK Fields

The ERR packet should also be authenticated, because otherwise an attacker could immediately close connection on either the client or the server side (DoS).


{#session-hijacking}
### Session Hijacking

Encryption and authentication were not required for the protocol and no other protocols are used below or on top of this protocol which would provide it for us. As mentioned above the connection ID must be authenticated using cryptographic signatures otherwise there is little to be done to prevent session hijacking if the connection ID becomes compromised. An attacker can easily take over the connection by using the not authenticated connection ID and responding faster than the expected communication partner.


### Privilege Escalation
SOFT protocol does not provide a specification to validate the file name that is sent by the client in the REQ packet. The file name can include any file path. An attacker can make use of this with a rooting attack: By adding a '/' as first character in a unix-like operating system, an insecure server implementation might provide access to the root directory of the file system. Moreover, the '../' pattern allows one to navigate through the servers file structure with ease. This behavior is especially dangerous if the application has root privileges.


Therefore, a secure server implementation must restrict file access, e.g. to only serve files from a specified directory.
The server should abort the connection with an Error *FILE\_NOT\_FOUND* if access to the file is denied.


### Replay Attacks

Although an attacker might not be able to create or modify packets, an attacker could replay packets traversing the network.

By replaying ACK packets an attacker can reduce the congestion window on the server. This can drastically reduce throughput.


{#future-work}
# Future Work
{#ip-layer-fragmentation}
## Considerations on IP Layer Fragmentation

The SOFT protocol has no control over the IP layer beneath the UDP layer. But decisions in the SOFT handshake have consequences on IP packet routing. In general, fragmentation on the IP layer can have significant effect on the robustness of SOFT packet transfer. Using IPv4, a router in the network can fragment the IP packet into smaller IP packets. There is no reliability mechanism on the IP layer, meaning a lost fragment is not resent. As a consequence, the assembled UDP packet will be incomplete and thus UDP silently drops the packet after the checksum computation of the packet. For IPv6, fragmentation and reassembling is only done by the communication endpoints. Thus, IPv6 packets that are too big for a router to be forwarded, are dropped.


This SOFT protocol version already provides the possibility to avoid IP layer fragmentation for IPv4 in general. Since 576 bytes is the minimum MTU size IPv4 hosts must support, the SOFT server can always set the upper limit of the maximum packet size to 496 bytes when a REQ packet with client's maximum packet size comes in.

Furthermore, the SOFT protocol has a rudimentary feature for the client to probe the maximum SOFT packet size. With the MPS inside the REQ packet of the handshake, the server may agree with this value and start to send DATA packets with this MPS length after completion of the handshake. If the client then receives fragmented DATA packets or does not receive any DATA packets at all, it may assume that this is due to a too high MPS value that results in IP fragmentation problems.


A future version of the SOFT protocol may avoid IP layer fragmentation issues by supporting path MTU discovery. For the client, a path MTU discovery can be started before sending the REQ packet. For the server, the discovery to the client would be done after receiving the REQ packet. However, this introduces complexity into the SOFT protocol since the server must notify the client that it is conducting a discovery so that the client does not start to retransmit the REQ packet due to an assumed timeout. Moreover, path MTU discovery from both sides would also be necessary again after a connection migration happened.

{#ipv6-support}
## Considerations on IPv6 Support

For IPv6 support, the maximum byte length of the file name in the REQ packet MUST be adapted to the minimal IPv6 packet size network hosts must support. Referring to [@RFC2460] that sets this value to 1280 bytes and defines that each IPv6 header has 40 bytes, the maximum byte length for a file name is the following:

~~~
(1280 - 40 (IPv6 Header) - 20 (UDP Datagram Header) - 12 (SOFT REQ Header)) byte = 1208 byte
~~~

Note: The calculation above does not take IPv6 extension headers into account.

Generally, the SOFT protocol could be encapsulated in an IPv6 packet, but the usage of extension headers might lead to fragmentation and the transport cannot be guaranteed.

{#request-cookies}
## Request Cookies

In the current version of the protocol, a server has to create a connection state and thereby consumes resources upon reception of a REQ packet. Clients might use the REQ in order to check if a file has changed without following it up with a transfer. Furthermore, this early state creation makes the protocol vulnerable to IP address spoofing and can, as mentioned in (#dos), lead to exhaustion of the server's resources and thereby denied access to legitimate clients.

To avoid this, future versions might introduce REQ cookies similar to TCP SYN-Cookies to postpone the state creation on the server and mitigate the DOS potential of the handshake. For such a mechanism, some state information could be encoded into the Connection ID (4 byte) field of the ACC packet, which is echoed back by the client in the subsequent ACK packet anyways. If these 4 bytes are not enough to store the state, some integrity information and an unique identifier, some extra fields for more sophisticated cookies have to be introduced in a future version.

{#client-adaptive-rtt}
## Client Adaptive RTT

In the current protocol version the client is not required to adapt its initial RTT value to changing environmental conditions. This should be addressed in a future version due to its strong effect on connection performance especially with regards to connection migration. A static RTT can result in very poor performance after connection migration due to the potentially changed network conditions effectively rendering the old RTT value useless.

One option to retrieve new RTT sames could be the use of spin bits.


{backmatter}

<reference anchor="assignment-1">
    <front>
        <title>Assignment 1: Robust File Transfer</title>
        <author initials="J." surname="Ott" fullname="J. Ott">
            <organization/>
        </author>
        <author initials="L." surname="Tonetto" fullname="L. Tonetto">
            <organization/>
        </author>
        <author initials="M." surname="Kosek" fullname="M. Kosek">
            <organization/>
        </author>
        <date year="2021"/>
        <abstract>
            <t/>
        </abstract>
    </front>
</reference>