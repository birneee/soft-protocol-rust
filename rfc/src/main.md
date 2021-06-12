%%%
title = "The Concrete Protocol (Draft)"
abbrev = "The Concrete Protocol"
ipr= "trust200902"
area = "Transport"
workgroup = "Group 10"
submissiontype = "IETF"
keyword = ["FTP"]
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
organization = "TUM"

[[author]]
initials = "T."
surname = "Midek"
fullname = "Thomas Midek"
organization = "TUM"

[[author]]
initials = "V."
surname = "Giridharan⁩"
fullname = "Vyas Giridharan⁩"
organization = "TUM"
%%%

.# Abstract
The Concrete Protocol is a protocol to enable robust file transfers of single or multiple files over the network encapsulated in UDP datagrams.
This document specifies the multilayered approach that includes a transport layer that is realized by *Concrete TCP* (*cTCP*) and a file transfer layer realized by *Concrete FTP* (*cFTP*). 

{mainmatter}

# Introduction

The *Concrete Protocol* is specifically developed to be robust, meaning that a successful file transfer has to be achieved in a variety of network conditions, with different device types, and/or file sizes.
The transfer has to be adequately efficient for different scenarios.
In this document security is considered only briefly, because this does not have high priority for the current version's specification.

## Terminology

The keywords **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL**, when they appear in this document, are to be interpreted as described in [@RFC2119].

## Objectives
The proposed protocol will have the following properties:

- The protocol **MUST** be UDP-based
- The protocol **MUST** be able to recover from connection drops
- The protocol **MUST** support connection migration, if the client's IP changes
- The protocol **MUST** support flow control
- The protocol **MUST** realize a minimal congestion control
- The protocol **MUST** utilize checksums for file integrity
- The protocol **SHALL** be able to transfer multiple files per connection
- The protocol **SHALL** be able to transfer specific parts of a file
- The server **SHALL** support multiple client connections
- The connection **SHALL** be migratable if the client's or server's IP or UDP port changes
- The protocol **SHALL** be efficient with small and large files (1 Byte upto 18 Exabytes)
- The protocol **SHALL** support large file names and paths in the request (1 MB)

Additional measures regarding efficiency and security are not the purpose of this initial draft and are subject for future work.


# The Layer Model

The protocol is split into two layers that each serve a specific purpose, namely the network layer and the data layer. The network layer is realized by *cTCP* which utilized *ACKs* to achieve flow control, congestion control, retransmission of packets and skip forward (see (#skip-forward)). The data layer is realized via *cFTP* which treats the overlaying *cTCP* connection as a continuous stream and handles the file requests in order to send single/multiple files.

All *cFTP* packets are encapsulated in a *cTCP* packet. 
All *cTCP* packets are encapsulated by a standard *UDP* packet.
The *UDP* packet can be encapsulated in an *IPv4* or *IPv6* packet.
The *UDP* port and the *IP* address information is required for connection migration of *cTCP*.

{align="center"}
~~~
:                       ...                     : 
+-----------------------------------------------+
|                IPv4 / IPv6 Packet             |
+-----------------------------------------------+
|                  UDP Datagram                 |
+ ----------------------------------------------+
|               Concrete-TCP Packet             |
+-----------------------------------------------+
|               Concrete-FTP Packet             |
+-----------------------------------------------+
~~~

As default a *UDP* datagram payload size of 512 Bit is chosen but this could be changed depending on the specific implementation, or dynamically determined. This would be a subject for future work.

# Concrete TCP (cTCP)

The *cTCP* part of the concrete protocol handles all networking-related tasks. It initializes the connection with the opposing connection partner and handles connection migrations, congestion and flow control as well as packet retransmission.

{#packet-types}
## Packet Types

{align="center"}
~~~
+-------+----- -+-------------------------------+
| Type  | Code  | Description                   | 
+ ------+-------+-------------------------------+
| INIT  | 0     | initialize connection         |
|       |       |                               |
| ACC   | 1     | accept connection             |
|       |       |                               |
| DATA  | 2     | data transport                |
|       |       |                               |
| ACK   | 3     | acknowledge/request/skip date |
|       |       |                               |
| FIN   | 4     | abort connection              |
|       |       |                               |
|       | 5-255 | reserved                      |
+-------+-------+-------------------------------+
~~~

## Encoding

{align="center"}
~~~
 0               1               2               3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|   Version=1   |  Packet Type  |                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+                               |
|                                    Packet Type Dependent      |
:                                                               :
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
~~~

The fields *Version*, *Packet Type*, *Status Code*, *Connection ID*, *Byte Index*, and *Receive Window Size* are encoded as Big-Endian unsigned integer numbers.
The field *Version* is always 1, for the current version's specification.
The field *Packet Type* must be one of the values from the table in (#packet-types).
The rest of the packet's content depends on the value of the *Packet Type* field.
Reserved fields should not be considered by implementations of the prococol.

{#field-sizes}
### Field Sizes

{align="center"}
~~~
+---------------------+----------+
| Field               | Size     |
+---------------------+----------+
| Version             | 1 Byte   |
|                     |          |
| Packet Type         | 1 Byte   |
|                     |          |
| Byte Index          | 8 Byte   |
|                     |          |
| Data                | variable |
|                     |          |
| Receive Window Size | 4 Byte   |
|                     |          |
| Status Code         | 1 Byte   |
+---------------------+----------+
~~~

For reasons of byte alignment and simplicity a field size of 1 Byte is chosen for *Version*, *Packet Type* and *Status Code* fields. The size of the *Byte Index* field is set to 8 Byte due to the ability to index 18 Exabytes of data which enables the transmission of multiple very large files. The size of the *Data* field is defined by the size of the encapsulating *UDP* datagram. The *Receive Windows Size* field size of 4 Bytes is chosen to be sufficient to signal a free buffer size of up to 4GB which is assumed to be enough.

### INIT Packet

{align="center"}
~~~
 0               1               2               3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|   Version=1   | Packet Type=0 |            Reserved           |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Connection ID                         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
~~~

The initiator of the connection chooses a random *Connection ID*.

### ACC Packet

{align="center"}
~~~
 0               1               2               3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|   Version=1   | Packet Type=1 |            Reserved           |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Connection ID                         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
~~~

This packet is sent as a response to a *INIT* packet to accept the connection. The *Connection ID* must be the same as the field in the *INIT* packet.

### DATA Packet

{align="center"}
~~~
 0               1               2               3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|   Version=1   | Packet Type=2 |            Reserved           |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Connection ID                         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                                                               |
+                Byte Index (Sequence Number)                   +
|                                                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                             Data                              |
:                                                               :
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
~~~

### ACK Packet

*ACK* packets serve two purposes as they not only request the next byte that should be sent but also acknowledges the reception of all previous bytes.

{align="center"}
~~~
 0               1               2               3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|   Version=1   | Packet Type=3 |            Reserved           |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Connection ID                         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                                                               |
+                Byte Index (Sequence Number)                   +
|                                                               |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                     Receive Window Size                       |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
~~~

### FIN Packet

{align="center"}
~~~
 0               1               2               3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|   Version=1   | Packet Type=4 |  Status Code  |   Reserved    |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                         Connection ID                         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
~~~

The field *Status Code* must be one of the values from the table in (#status-codes).


{#status-codes}
## Status Codes

{align="center"}
~~~
+--------------------+-------+-------------------------------+
| Name               | Code  | Description                   |
+--------------------+-------+-------------------------------+
| EOF                | 0     | end of file (end of stream)   |
|                    |       |                               |
| UNKNOWN_CONNECTION | 1     | unknown connection id         |
|                    |       |                               |
| TAKEN_CONNECTION   | 2     | taken connection id           |
|                    |       |                               |
|                    | 3-255 | reserved                      |
+--------------------+-------+-------------------------------+
~~~

## Connection Flow

A connection is split into two parts, first the connection setup phase and second the data flow phase.

### Setup

In the setup phase the connection partners check for equal protocol version numbers and the initiator proposes a random identifier as the *Connection ID*. 

{align="center"}
~~~
Client                      Server
   |                           |
   | ----------INIT----------> |
   |  |cTCP Protocol Version|  |
   |  |Connection ID        |  |
   |                           |
   | <---------ACC------------ |
   |      |Connection ID|      |
   |                           |
   | <---------DATA----------> | The data exchange is further detailed
   |                           | in the following section
   | ----------FIN-----------> |
   |      |Connection ID|      |
   |      |Status       |      |
   |                           |
~~~

The *FIN* packet can be sent by the client or the server.

### Data Flow

The data flow phase is where the actual data transfer happens and is indicated by the transmission of *ACK* and *DATA* packets to the opposing side.

Note that the connections are symmetrical i.e. the server and client can both send and acknowledge data. 
This is also good for reusing the code implementation on both client and server.

The purpose of the *ACK* in not only to acknowledge received data, but also to request the next byte index that one wants to receive (see also (#skip-forward)).

Also, the actual exchange after the initial handshake phase above is started by the Server since he will request the *Request Catalog File*. 
Therefore, the first packet will be an *ACK* by the Server.

{align="center"}
~~~
Client                       Server
   |                            |
   | <----------ACK------------ | Server requests first byte of RCF
   |     | Connection ID   |    |
   |     | Next Byte Index |    |
   |     | Window Size     |    |
   |                            |
   | -----------DATA----------> |
   |      | Connection ID |     |
   |      | Byte Index    |     |
   |      | Data          |     |
   |      | Checksum      |     |
   |                            |
   | <----------ACK------------ |
   |     | Connection ID   |    |
   |     | Next Byte Index |    |
   |     | Window Size     |    |
   |                            |
   |           ...              | 
   |                            |
   | -----------ACK-----------> | Client now starts requesting
   |     | Connection ID   |    | bytes from FRF
   |     | Next Byte Index |    |
   |     | Window Size     |    |
   |                            |
~~~

## Connection Migration

Each connection has a unique *Connection ID* which is associated to a remote *IP* and *UDP* port. The *Connection ID* of each packet is used to correlate it with the endpoint to which the data needs to be sent.
The *IP* and *UDP* port number of the client is updated every 15 minutes to handle connection migrations.
*Concrete protocol* connections are not strictly bound to a single network path.

The Client that initiates the connection migration is called the connection migration initiator. The Server is the connection migration responder. Connection migration uses the *Connection ID* to allow connections to transfer to a new network path. The use of a *Connection ID* allows connections to survive changes to endpoint addresses (IP address and port) such as those caused by an endpoint migrating to a new network.

The *Connection ID* is a 32 bit randomly generated ID.

Connections are maintained by the server for a minute and then removed if no packets are received.

### Example

~~~
Client                              Server
   |                                  |
   | ----------cTCP Packet----------> |
   |  Source IP=116.0.0.1             |
   |  Destination IP=156.0.0.10       | 
   |  Source UDP Port=36004           |
   |  Destination UDP Port=9840       | 
   |  cTCP Connection ID=16284676044  |
   |                                  | ! Client IP changes 
   |                                  | from 116.0.0.1 to 116.0.0.2
   |                                  | and UDP Port changes
   |                                  | from 36004 to 36005
   |                                  |
   | X------- -cTCP Packet----------- | Server sents to old IP/Port
   |  Source IP=156.0.0.10            |
   |  Destination IP=116.0.0.1        | *outdated
   |  Source UDP Port=9840            |
   |  Destination UDP Port=36004      | *outdated
   |  cTCP Connection ID=16284676044  |
   |                                  |
   | ----------cTCP Packet----------> | 
   |  Source IP=116.0.0.2             | *new 
   |  Destination IP=156.0.0.10       | 
   |  Source UDP Port=36005           | *new
   |  Destination UDP Port=9840       | 
   |  cTCP Connection ID=16284676044  | Server recognizes the
   |                                  | Connection ID and updates
   |                                  | the IP and Port
   |                                  |
   | <----------cTCP Packet---------- | Server sends to updates IP/Port
   |  Source IP=156.0.0.10            |
   |  Destination IP=116.0.0.2        | *updated 
   |  Source UDP Port=9840            |
   |  Destination UDP Port=36005      | *updated 
   |  cTCP Connection ID=16284676044  |  
   |                                  |
~~~

Note that it does not matter if the server or client IP changes as the migration process works the same in both cases. As connections are identified by the *Connection ID*, packets can easily be associated with the corresponding connection partner. Therefore, when a packet is received from a new IP but contains the *Connection ID* of another connection, it is simply assumed that the IP of the communication partner has changed and the IP is updated accordingly. This is also a security issue that could be addressed in future work.

Note that if Server and Client IPs change at the same time, connection migration is not possible because they are not able to find each other.

## Congestion & Flow Control

The initial *RTT* is choosen to be 3 seconds. The *RTT* is then updated using a moving average:

{align="center"}
~~~
RTT = gamma * oldRTT + (1-gamma) * newRTTSample

~~~

with *gamma* denoting a constant weighting factor.

The new *RTT* samples are obtained as the time measured between a packets transmission and the reception of its acknowledgment.

For congestion controll additive-increase/multiplicative-decrease *AIMD* is used.
Congestion is detected if 3 duplicate *ACKs* are received which leads to a decrease of the congestion window. 
The congestion window is updated every RTT as follows:

{align="center"}
~~~
Let w(t) bet the congestion window size at time t:

w(t+1) = w(t) + alpha    if no congestion is detected
w(t+1) = w(t) * beta     if congestion is detected

~~~

The initial congestion window size is set to one *cTCP* packet size. 
The additive increase factor *alpha* is chosen to be one *cTCP* packet size. 
The multiplicative decrease factor *beta* is chosen as 1/2 which results in halfing the congestion window if congestion is detected.

The decision to transmit is then determined by the following formula:

{align="center"}
~~~
MaxWindow = min{AdvertisedWindow, CongestionWindow}
EffectiveWindow = MaxWindow - (LastByteSent - LastByteAcked)

~~~

with the *EffectiveWindow* indicating how much data can be sent. 
The *MaxWindow* is the maximum number of unacknowledged data allowed in circulation. 
The *AdvertisedWindow* is sent by the opposing endpoint in every *ACK* packet and is comprised of the receive buffer size.

If the *EffectiveWindow* is greater than 0 more data can be transmitted.

{#skip-forward}
## Skip Forward

*Skip Forward* is a feature for the purpose of skipping the transmission to a certain byte index in the stream. 
This is mainly utilized when continuing a connection after e.g. connection drops or if a connection is aborted on purpose.
The client does not have to receive the entire byte stream, it can choose to skip at any point during the connection.
In order to *Skip Forward* the client transmits an *ACK* Packet containing the byte index at which the transmission shall be continued. 
This can be any byte index which therefore enables clients to only choose to receive certain byte ranges within a file.

### Use Cases
The *Skip Forward* feature allows a client to receive only certain parts of files, which is useful in many scenarios.

*The Concrete Protocol* can be used in multimedia apps for audio/video streaming, where each audio/video file is seekable and can be started from a certain position.
It also has applications in databases. If the exact position of the data inside a large file is known, the client can fetch only a specific part of the data.
Like mentioned before *Skip Forward* is also used to resume a closed data transfer, e.g. the client can resume the download of a large file on another day.

### Example Flow

In the following example the server sends 512 bytes payload per *cTCP* data packet.

{align="center"}
~~~
Client                       Server
   |                           |
   | <----DATA<index=0>------- |
   | <----DATA<index=512>----- |
   | <----DATA<index=1024>---- |
   |                           | 
   |  ----ACK<index=1536>----> | Client acknowledges bytes until 1536
   |                           | 
   | <----DATA<index=1536>---- | Server continues nominally
   | <----DATA<index=2048>---- |
   | <----DATA<index=2560>---- |
   |                           |
   |  ----ACK<index=9000>----> | Client skips forward to byte 9000
   |                           |
   | <----DATA<index=8704>---- |
   |                           |
~~~

The protocol uses cumulative *ACKs* meaning that it will just send one *ACK* to acknowledge all packets with a byte index smaller than the index in the *ACK*. An example of this can be seen in the above figure where packets with index 0, 512 and 1024 are acknowledged by the *ACK* requesting index 1536.

The client can send *ACKs* requesting any byte index. If the index is larger than the file size the server will respond with a *FIN* packet containing with status code *EOF*. The byte index does not have to be aligned on the client side since the server will realign the data to 512 Byte. If the server detects an *ACK* with a non 512 Byte aligned index it sends a packet with content size smaller than 512 Byte to realign the transmission to 512 Byte per packet.

### No Skip Backward
Skipping backwards is not supported. 
If a receiver sends an *ACK* with a byte index before an index that has already be acknowledged, the sender does simply ignore that *ACK* packet.
Therefore it is only possible to read sequentially or skip in the forward direction.

## Connection Timeout
If an endpoint does not receive a packet from the communication partner after 5 seconds, it will start to retransmit packets. If no packets are received for a specificied amount of time the connection is then completly dropped. This means that the state information is cleaned up on the endpoint and the endpoint does no longer associate a connection with this connection id. The connection drop timeout can be configured by the client and server individually. We recommend a default timeout of 60 seconds. The retransmission timer of 5 seconds is chosen but this could also be later calculated using *RTT* information.

Due to the Skip Forward feature, the resumption of a file transfer is possible and is handled by the *cFTP* protocol (see (#recovery-resumption)).

# Concrete FTP (cFTP)

From the perspective of *cFTP* the *cTCP* communication is abstracted as a continuous stream.

After the *cTCP* connection is initialized the client is sending a *UTF-8* encoded Request Catalog File (*RCF*) that contains an ordered listing of the files that are requested.

Upon receiving this *RCF* a server will respond with a Fat Response File (*FRF*) which will contain status information as well as the concatenated files itself. The status information consists of:

- File status (see (#file-status-codes))
- File size (in Bytes)
- File checksum (SHA1)

{#file-status-codes}
## File Status Codes

{align="center"}
~~~
+--------------------+-----------------------------------------+
| Code               | Description                             |
+--------------------+-----------------------------------------+
| OK                 | file is ready to be transferred         |
|                    |                                         |
| NOT_FOUND          | file not found                          |
|                    |                                         |
| ACCESS_DENIED      | access denied                           |
|                    |                                         |
| CHECKSUM_NOT_READY | the file checkum is not calculated yet, |
|                    | the client has to retry later           |
|                    |                                         |
+--------------------+-----------------------------------------+
~~~

## Data Flow

The data flow of *cFTP* is very simple.
When a *cTCP* connection is established, the Client sends a *RCF* to the server to which the server responds with a *FRF*. After *FRF* transfer completion the connection is closed.

{align="center"}
~~~
Client                               Server
   |                                   |
   | ------Request Catalog File------> |     
   |     | File Name(s) / Path(s) |    |
   |                                   |
   | <--------Fat Response File------- |
   |        | Protocol Version |       |
   |        | File Status Code |       |
   |        | File Size        |       |
   |        | File Checksum    |       |
   |        | File Content     |       |
   |                                   |
~~~

## Encoding
The *cFTP* protocol is a combination of text based and binary format.
The *cFTP* requests and responses are generally text based (inspired by *HTTP*) which makes it human-readable and easy to understand.
All text is encoded in *UTF-8*.
The exception is the file data itself, because it is not very efficient to transfer binary data encoded as text.
The file data is transfered as raw binary data.
Therefore the *FRF* consists of two parts: at first the *UTF-8* encoded header information and then the raw binary data.
The two parts are separated by two newline symbols (UTF-8: U+000A, \n).

### Request Catalog File (RCF)

The Request Catalog File contains the protocol version to be used as well as the names or paths of the files to be transmitted. It is exclusively sent by the entity that wants to receive files.

#### Example

{align="left"}
~~~
cFTPv1
faust.txt
/ect/shadow
yellow_submarine.mp3
HL3.exe

~~~

{#frf}
### Fat Response File (FRF)

The *FRF* is a combination of text and binary file.
The first *UTF-8* encoded header part contains the protocol and version identifier (*cFTPv1*), and for each requested file, a status code, the file size in bytes and the *SHA1* checksum.
This header information is separated by space (UTF-8: U+0020) and newline (UTF-8: U+000A, \n) symbols.

The end of header is indicated by two consecutive newline symbols (UTF-8: U+000A, \n).
Directly after that the binary data starts, which is just the unmodified raw content of the files.
If multiple files are transfered the data is simply concatenated, therefore the client has to split the received data blob based on the file size information from the header.

The SHA1 checksum is encoded as an hexadecimal character string.

{#frf-examples}
#### Examples

The following example shows a response, when all files are ready to be transfered.

{align="left"}
~~~
cFTPv1
OK 568500 6d851350e056b44cba849ee1bb76e7391b93eb12
OK 2000 a94a8fe5ccb19ba61c4c0871bb76e7391b93eb66
OK 6830025 9ba61c4c087a61c4cd849ee1bb76e7391b93eb32
OK 33333333333 e056b4ba61c4c087ad849ee1bb76e7391b93eb33

\x46\x61\x75\x73\x74\x3a\x20\x44\x65\x72\x20\x54\x72\x61...
~~~

In the following response some files are not ready for transfer.
Notice that if not all file status are *OK* not actual file data is transfered.
The connection is closed after *FRF* header and two trailing newline symbols are transfered.

{align="left"}
~~~
cFTPv1
OK 568500 6d851350e056b44cba849ee1bb76e7391b93eb12
ACCESS_DENIED 2000 a94a8fe5ccb19ba61c4c0871bb76e7391b93eb66
OK 6830025 9ba61c4c087a61c4cd849ee1bb76e7391b93eb32
CHECKSUM_NOT_READY 33333333333

~~~

## File Checksums
The response from the server contains a SHA1 checksum for every requested file (see (#frf)).
When the client has received one entire file, it can calculate the checksum and compare it to the checksum received from the server.
That is how the client can validate if the received file was correctly received.
Furthermore a changed checksum is an indicator for the client that the file has changed since a previous transfer.
For server implementations we recommend to cache or calculate the file checksums beforehand, because this can take some time for larger files, and is not sufficient to do on the fly, when the files are requested.
If a checksum is not ready to send, the server can respond with the file status CHECKSUM\_NOT\_READY, as seen in the following example (see (#frf-examples)).

{#recovery-resumption}
## Recovery & Resumption

If the connection is interrupted for some reason, the client can resume the data transfer through the following procedure:

- Client sends identical Request Catalog File (RCF) to server
- Server sends Fat Response File (FRF) containing current file status
- Client validates checksums for file changes
- Client sends ACK with byte index (see (#skip-forward))

In order to resume the file transfer the client has to remember the content of the *RCF*, the checksums, and the byte index where it left off.
By comparing the old checksums with the new ones, the client can detect changed files.
A possible measure of the client is then to receive the changed files again from the beginning.

### Example

{align="center"}
~~~
Client                               Server
   |                                   |
   | <----------cTCP DATA------------- |
   |                                   | Transfer from byte until 8704
   | -------cTCP ACK<index=8704>-----> |
   |                                   |
   |-----------------------------------|
   |       Connection Interrupt        | Connection is interrupted
   |-----------------------------------|
   |                                   |
   | <--------cTCP Handshake---------> | Connection is reestablished
   |                                   |
   | ----cFTP Request Catalog File---> | Client sends previous RCF
   |     | File Name(s) / Path(s) |    |
   |                                   |
   | <-----cFTP Fat Response File----- | Server sends updated FRF
   |        | Protocol Version |       | with updated contents
   |        | File Status Code |       | like the current checksum
   |        | File Size        |       |
   |        | File Checksum    |       |
   |        | File Content     |       |
   |                                   |
   | ------cTCP ACK<index=8704>------> | Client sends ACK containing 
   |                                   | byte index 8704 to skip
   |                                   | forward and resume transfer
   |                                   |
~~~
See also (#skip-forward).

Note that a connection interruption only means cases where the interruption is longer than the *cTCP* timeout, or the connection is aborted by some reason which leads to the server or client states being cleaned up. If this is not the case than there is no need for a new connection setup and communication can be simply resumed.

# Security
This section contains some hints on the security of the protocol, but as already mentioned security is not considered for this version of the specification.

## Availability
Currenty no measures are taken against Denial-of-service-attack.
Due to the server having to complete much more work than a client through, for example, checksum calculations and harddisk accesses, there exists a vulnerability against *DoS* attacks.

## Integrity
Due to the random *Connection ID* being a 32bit integer there is a possibility of identifiers clashing, e.g. other endpoint already has a connection with the same identifier. This could potentially be used for the man-in-the-middle attacks.

Also, the connection itself is not protected against manipulation as there is no integrity check via cryptographic signatures.
Although the transferred files do have a checksum that can be validated, these checksums can be manipulated too.

## Confidentiality
The protocol does not handle encryption, therefore the transferred data can easily be eavesdropped.

{backmatter}